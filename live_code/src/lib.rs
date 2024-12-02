#[allow(unused_imports)]
use axum::{
    body::Bytes, extract::Path, extract::State, http::StatusCode, response, routing::post, Json,
    Router,
};
use nih_plug::prelude::*;
use rtrb::{Consumer, PopError, Producer, RingBuffer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::oneshot;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Note {
    note_num: u8,
    dur_ms: i64,
    velocity: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum EventType {
    Rest,
    NoteEvent(Note),
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Event {
    action: EventType,
    dur_beats: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pattern {
    events: Vec<Event>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NamedPattern {
    name: String,
    events: Vec<Event>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum NoteType {
    On,
    Off,
    Rest,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SimpleNoteEvent {
    note_type: NoteType,
    timing: u32,
    voice_id: Option<i32>,
    channel: u8,
    note: u8,
    velocity: f32,
}

#[derive(Clone)]
struct PrecisePattern {
    events: HashMap<usize, Vec<SimpleNoteEvent>>,
    length_samples: usize,
    playing: bool,
}

impl PrecisePattern {
    fn from(pattern: Pattern, sample_rate: f32, tempo: f64, playing: bool) -> PrecisePattern {
        let samples_per_beat = (sample_rate as f64 * (60.0 / tempo)) as usize;
        let samples_per_milli = sample_rate / 1000.0;
        let mut sample_idx: usize = 0;
        let mut events_map: HashMap<usize, Vec<SimpleNoteEvent>> = HashMap::new();

        for event in pattern.events {
            match event.action {
                EventType::NoteEvent(note) => {
                    events_map.insert(
                        sample_idx,
                        vec![SimpleNoteEvent {
                            note_type: NoteType::On,
                            timing: sample_idx as u32,
                            voice_id: None,
                            channel: 1,
                            note: note.note_num,
                            velocity: note.velocity,
                        }],
                    );
                    let note_off_timing = (sample_idx
                        + ((samples_per_milli as f64) * (note.dur_ms as f64)) as usize)
                        as u32;

                    events_map.insert(
                        note_off_timing as usize,
                        vec![SimpleNoteEvent {
                            note_type: NoteType::Off,
                            timing: note_off_timing,
                            voice_id: None,
                            channel: 1,
                            note: note.note_num,
                            velocity: 0.0,
                        }],
                    );
                }
                EventType::Rest => {
                    events_map.insert(
                        sample_idx,
                        vec![SimpleNoteEvent {
                            note_type: NoteType::Rest,
                            timing: 0,
                            voice_id: None,
                            channel: 1,
                            note: 0,
                            velocity: 0.0,
                        }],
                    );
                }
            }
            sample_idx += ((samples_per_beat as f64) * event.dur_beats) as usize;
        }
        return PrecisePattern {
            events: events_map,
            length_samples: sample_idx,
            playing: playing,
        };
    }

    fn get_events(&self, start: usize, end: usize) -> Vec<SimpleNoteEvent> {
        if self.length_samples == 0 || !self.playing {
            return vec![];
        }
        let adj_start = start % self.length_samples;
        let adj_end = end % self.length_samples;

        if adj_end < adj_start {
            let mut pat_end = self.get_events_adj(adj_start, self.length_samples - 1, 0);
            let next_pat = self.get_events_adj(0, adj_end, self.length_samples - adj_start);
            pat_end.extend(next_pat);
            return pat_end;
        }
        return self.get_events_adj(adj_start, adj_end, 0);
    }

    fn get_events_adj(
        &self,
        adj_start: usize,
        adj_end: usize,
        timing_offset: usize,
    ) -> Vec<SimpleNoteEvent> {
        let mut note_events: Vec<SimpleNoteEvent> = Vec::new();

        for sample_index in (adj_start..adj_end).map(|x| x as usize) {
            match self.events.get(&sample_index) {
                None => (),
                // Need to adjust the timing so that it is relative to the current audio buffer.
                Some(events) => {
                    note_events.extend(events.into_iter().cloned().map(|ev| SimpleNoteEvent {
                        note_type: ev.note_type,
                        timing: ((ev.timing as usize) - adj_start + timing_offset) as u32,
                        voice_id: ev.voice_id,
                        channel: ev.channel,
                        note: ev.note,
                        velocity: ev.velocity,
                    }))
                }
            };
        }
        note_events
    }
}

pub struct Live {
    params: Arc<LiveParams>,
    playing: bool,
    patterns: HashMap<String, Pattern>,
    precise_patterns: HashMap<String, PrecisePattern>,

    // Plugin thread and command thread will communicate using these.
    commands_rx: Option<Consumer<Command>>,
    responses_tx: Option<Producer<Command>>,

    // Command thread will be shutdown by the plugin thread using this.
    shutdown_tx: Option<oneshot::Sender<()>>,

    tempo_prev_cycle: Option<f64>,
}

impl Live {
    fn play(
        &mut self,
        buffer: &mut Buffer,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let transport = context.transport();
        let pos_samples = transport.pos_samples().unwrap_or(0) as usize;

        if let Err(_) = self.process_commands(context) {
            return ProcessStatus::Error("error processing commands");
        }
        let transport = context.transport();
        let tempo = transport.tempo.unwrap_or(120.0);

        if tempo != self.tempo_prev_cycle.unwrap_or(120.0) {
            nih_log!("recomputing patterns after tempo change");
            self.recompute_patterns(context);
        }
        for precise_pattern in self.precise_patterns.values() {
            if !precise_pattern.playing {
                continue;
            }
            for event in precise_pattern.get_events(pos_samples, pos_samples + buffer.samples()) {
                self.send(context, event);
            }
        }
        ProcessStatus::Normal
    }

    fn recompute_patterns(&mut self, context: &mut impl ProcessContext<Self>) {
        let transport = context.transport();

        for (name, pattern) in self.patterns.iter() {
            let mut playing = false;
            if let Some(existing_pattern) = self.precise_patterns.get(name) {
                playing = existing_pattern.playing;
            }
            let precise_pattern = PrecisePattern::from(
                pattern.clone(),
                transport.sample_rate,
                transport.tempo.unwrap_or(120.0),
                playing,
            );
            self.precise_patterns
                .insert(name.clone(), precise_pattern.clone());
        }
    }

    fn process_commands(
        &mut self,
        context: &mut impl ProcessContext<Self>,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(cmds) = self.commands_rx.as_mut() {
            match cmds.pop() {
                Ok(Command::PatternStart(pattern)) => self.start_pattern(context, pattern),
                Ok(Command::PatternStop(name)) => {
                    nih_log!("stopping pattern");
                    match self.precise_patterns.get(&name) {
                        Some(precise_pattern) => {
                            let mut clone = precise_pattern.clone();
                            clone.playing = false;
                            self.precise_patterns.insert(name.clone(), clone);
                            nih_log!("stopped pattern {}", name.clone());
                            Ok(())
                        }
                        None => {
                            nih_log!("no pattern with name {}", name);
                            Ok(())
                        }
                    }
                }
                // TODO: handle this command
                Ok(Command::PatternList(_)) => Ok(()),
                Err(PopError::Empty) => Ok(()),
            }
        } else {
            Ok(()) // Maybe panic here? This should be unreachable.
        }
    }

    fn start_pattern(
        &mut self,
        context: &mut impl ProcessContext<Self>,
        named_pattern: NamedPattern,
    ) -> Result<(), Box<dyn Error>> {
        let transport = context.transport();
        let precise_pattern = PrecisePattern::from(
            Pattern {
                events: named_pattern.events.clone(),
            },
            transport.sample_rate,
            transport.tempo.unwrap_or(120.0),
            true,
        );
        self.patterns.insert(
            named_pattern.name.clone(),
            Pattern {
                events: named_pattern.events.clone(),
            },
        );
        self.precise_patterns
            .insert(named_pattern.name.clone(), precise_pattern.clone());
        Ok(())
    }

    fn start(
        &mut self,
        buffer: &mut Buffer,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let transport = context.transport();

        self.playing = true;

        nih_log!(
            "starting live (sample rate {}) (tempo {})",
            transport.sample_rate,
            transport.tempo.unwrap_or(120.0),
        );
        self.play(buffer, context);

        return ProcessStatus::Normal;
    }

    fn send(&self, context: &mut impl ProcessContext<Self>, sev: SimpleNoteEvent) -> () {
        match sev.note_type {
            NoteType::On => context.send_event(NoteEvent::NoteOn {
                timing: sev.timing,
                voice_id: sev.voice_id,
                channel: sev.channel,
                note: sev.note,
                velocity: sev.velocity,
            }),
            NoteType::Off => context.send_event(NoteEvent::NoteOff {
                timing: sev.timing,
                voice_id: sev.voice_id,
                channel: sev.channel,
                note: sev.note,
                velocity: 0.0,
            }),
            NoteType::Rest => {}
        }
    }
}

#[derive(Params)]
struct LiveParams {}

impl Default for Live {
    fn default() -> Self {
        Self {
            params: Arc::new(LiveParams::default()),
            playing: false,
            // We need the tempo and sample rate to properly initialize this.
            // Will be done on the first process() call.
            patterns: HashMap::new(),
            precise_patterns: HashMap::new(),
            commands_rx: None,
            responses_tx: None,
            shutdown_tx: None,
            tempo_prev_cycle: None,
        }
    }
}

impl Default for LiveParams {
    fn default() -> Self {
        Self {}
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum Command {
    PatternList(Vec<String>),
    PatternStart(NamedPattern),
    PatternStop(String),
}

#[allow(dead_code)]
pub struct Controller {
    commands: Mutex<Producer<Command>>,
    responses: Mutex<Consumer<Command>>,
}

#[axum::debug_handler]
pub async fn start_pattern(
    State(controller): State<Arc<Controller>>,
    Path(pattern_name): Path<String>,
    Json(pattern): Json<Pattern>,
) -> response::Result<String, StatusCode> {
    let mut cmds = controller.commands.lock().unwrap();
    // TODO: handle when the queue is full
    let named_pattern = NamedPattern {
        name: pattern_name,
        events: pattern.events,
    };
    match cmds.push(Command::PatternStart(named_pattern)) {
        Ok(_) => Ok(String::from("ok")),
        Err(_err) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[axum::debug_handler]
pub async fn stop_pattern(
    State(controller): State<Arc<Controller>>,
    Path(pattern_name): Path<String>,
) -> response::Result<String, StatusCode> {
    let mut cmds = controller.commands.lock().unwrap();
    // TODO: handle when the queue is full
    match cmds.push(Command::PatternStop(pattern_name)) {
        Ok(_) => Ok(String::from("ok")),
        Err(_err) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

impl Plugin for Live {
    const NAME: &'static str = "LiveCode";
    const VENDOR: &'static str = "Brian Sorahan";
    const URL: &'static str = "https://youtu.be/dQw4w9WgXcQ";
    const EMAIL: &'static str = "info@example.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            // This is also the default and can be omitted here
            main_input_channels: None,
            main_output_channels: None,
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: None,
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_OUTPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let (commands_tx, commands_rx) = RingBuffer::<Command>::new(256); // Arbitrary buffer size
        let (responses_tx, responses_rx) = RingBuffer::<Command>::new(256); // Arbitrary buffer size
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let commands = Arc::new(Controller {
            commands: Mutex::new(commands_tx),
            responses: Mutex::new(responses_rx),
        });
        self.shutdown_tx = Some(shutdown_tx);
        self.commands_rx = Some(commands_rx);
        self.responses_tx = Some(responses_tx);

        thread::spawn(move || {
            let app = Router::new()
                .route("/start/:pattern_name", post(start_pattern))
                .route("/stop/:pattern_name", post(stop_pattern))
                .with_state(commands);

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
                    .await
                    .unwrap();
                axum::serve(listener, app)
                    .with_graceful_shutdown(async move { shutdown_rx.await.ok().unwrap() })
                    .await
                    .unwrap();
            });
        });
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let transport = context.transport();
        let tempo = Some(transport.tempo.unwrap_or(120.0));

        if transport.playing {
            if !self.playing {
                self.start(buffer, context);
            } else {
                self.play(buffer, context);
            }
        } else {
            self.playing = false;
        }
        self.tempo_prev_cycle = tempo;

        ProcessStatus::Normal
    }

    fn deactivate(&mut self) -> () {
        if let Some(sender) = self.shutdown_tx.take() {
            sender.send(()).expect("Failed to send shutdown signal");
        }
        nih_log!("deactivating...");
    }
}

impl ClapPlugin for Live {
    const CLAP_ID: &'static str = "net.sorahan.brian.live";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Trigger a midi note every beat");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect, ClapFeature::Utility];
}

impl Vst3Plugin for Live {
    const VST3_CLASS_ID: [u8; 16] = *b"XXXXXXXXXXXXXXXX";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Tools];
}

nih_export_clap!(Live);
nih_export_vst3!(Live);

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_precise_pattern() -> Result<(), String> {
        let pattern = Pattern {
            events: vec![
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 60,
                        velocity: 0.8,
                        dur_ms: 20,
                    }),
                    dur_beats: 1.0,
                },
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 96,
                        velocity: 0.8,
                        dur_ms: 20,
                    }),
                    dur_beats: 1.0,
                },
            ],
        };
        let sample_rate = 48000 as f32;
        let tempo = 110 as f64;
        let precise_pattern = PrecisePattern::from(pattern.clone(), sample_rate, tempo, true);
        let buffer_size_samples = 256 as usize;
        let expectations: HashMap<usize, Vec<SimpleNoteEvent>> = HashMap::from([
            (
                0,
                vec![SimpleNoteEvent {
                    note_type: NoteType::On,
                    timing: 0,
                    voice_id: None,
                    channel: 1,
                    note: 60,
                    velocity: 0.8,
                }],
            ),
            (
                3,
                vec![SimpleNoteEvent {
                    note_type: NoteType::Off,
                    timing: 192,
                    voice_id: None,
                    channel: 1,
                    note: 60,
                    velocity: 0.0,
                }],
            ),
            (
                102,
                vec![SimpleNoteEvent {
                    note_type: NoteType::On,
                    timing: 69,
                    voice_id: None,
                    channel: 1,
                    note: 96,
                    velocity: 0.8,
                }],
            ),
            (
                106,
                vec![SimpleNoteEvent {
                    note_type: NoteType::Off,
                    timing: 5,
                    voice_id: None,
                    channel: 1,
                    note: 96,
                    velocity: 0.0,
                }],
            ),
            (
                204,
                vec![SimpleNoteEvent {
                    note_type: NoteType::On,
                    timing: 138,
                    voice_id: None,
                    channel: 1,
                    note: 60,
                    velocity: 0.8,
                }],
            ),
        ]);
        for (bufnum, expected_events) in expectations.into_iter() {
            let buffer_start_samples = bufnum * buffer_size_samples;
            let buffer_end_samples = buffer_start_samples + buffer_size_samples;
            let events = precise_pattern.get_events(buffer_start_samples, buffer_end_samples);
            assert_eq!(expected_events, events);
        }
        Ok(())
    }
}
