#[allow(unused_imports)]
use axum::{body::Bytes, extract::State, http::StatusCode, response, routing::post, Json, Router};
use nih_plug::prelude::*;
use rtrb::{Consumer, PopError, Producer, RingBuffer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
pub struct Event {
    note: Note,
    dur_beats: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pattern {
    events: Vec<Event>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum NoteType {
    On,
    Off,
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
}

impl PrecisePattern {
    fn from(pattern: Pattern, sample_rate: f32, tempo: f64) -> PrecisePattern {
        let samples_per_beat = (sample_rate as f64 * (60.0 / tempo)) as usize;
        let samples_per_milli = sample_rate / 1000.0;
        let mut sample_idx: usize = 0;
        let mut events_map: HashMap<usize, Vec<SimpleNoteEvent>> = HashMap::new();

        for event in pattern.events {
            events_map.insert(
                sample_idx,
                vec![SimpleNoteEvent {
                    note_type: NoteType::On,
                    timing: sample_idx as u32,
                    voice_id: None,
                    channel: 1,
                    note: event.note.note_num,
                    velocity: event.note.velocity,
                }],
            );
            let note_off_timing = (sample_idx
                + ((samples_per_milli as f64) * (event.note.dur_ms as f64)) as usize)
                as u32;

            events_map.insert(
                note_off_timing as usize,
                vec![SimpleNoteEvent {
                    note_type: NoteType::Off,
                    timing: note_off_timing,
                    voice_id: None,
                    channel: 1,
                    note: event.note.note_num,
                    velocity: 0.0,
                }],
            );
            sample_idx += ((samples_per_beat as f64) * event.dur_beats) as usize;
        }
        return PrecisePattern {
            events: events_map,
            length_samples: sample_idx,
        };
    }

    fn get_events(&self, start: usize, end: usize) -> Vec<SimpleNoteEvent> {
        if self.length_samples == 0 {
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
    precise_patterns: HashMap<String, PrecisePattern>,

    // Plugin thread and command thread will communicate using these.
    commands_rx: Option<Consumer<Command>>,
    responses_tx: Option<Producer<Command>>,

    // Command thread will be shutdown by the plugin thread using this.
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Live {
    fn play(
        &mut self,
        buffer: &mut Buffer,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let transport = context.transport();
        let pos_samples = transport.pos_samples().unwrap_or(0) as usize;

        // It's feasible that the plugin's internal time could drift from the host's
        // We aren't really sure exactly how many samples the host considers 1 bar to be!
        // Maybe we can improve the synchronization of our events with events being played in the host?
        // Here is a snippet of what the info printed by the log line looks like.
        //
        // 12:44:53 [INFO] pos_beats 3.9111111112870276 bar_start_pos_beats 0 bar_number 0
        // 12:44:53 [INFO] pos_beats 3.9306666664779186 bar_start_pos_beats 0 bar_number 0
        // 12:44:53 [INFO] pos_beats 3.950222222134471 bar_start_pos_beats 0 bar_number 0
        // 12:44:53 [INFO] pos_beats 3.9697777777910233 bar_start_pos_beats 0 bar_number 0
        // 12:44:53 [INFO] pos_beats 3.9893333334475756 bar_start_pos_beats 0 bar_number 0
        // 12:44:53 [INFO] pos_beats 4.008888889104128 bar_start_pos_beats 4 bar_number 1
        // 12:44:53 [INFO] pos_beats 4.028444444295019 bar_start_pos_beats 4 bar_number 1
        // 12:44:53 [INFO] pos_beats 4.047999999951571 bar_start_pos_beats 4 bar_number 1
        // 12:44:53 [INFO] pos_beats 4.0675555556081235 bar_start_pos_beats 4 bar_number 1
        // 12:44:53 [INFO] pos_beats 4.087111111264676 bar_start_pos_beats 4 bar_number 1
        // nih_log!(
        //     "pos_beats {} bar_start_pos_beats {} bar_number {}",
        //     transport.pos_beats().unwrap_or(0.0),
        //     transport.bar_start_pos_beats().unwrap_or(0.0),
        //     transport.bar_number().unwrap_or(0),
        // );
        if let Some(precise_pattern) = self.get_current_pattern(context) {
            for event in precise_pattern.get_events(pos_samples, pos_samples + buffer.samples()) {
                self.send(context, event);
            }
        }
        ProcessStatus::Normal
    }

    fn get_current_pattern(
        &mut self,
        context: &mut impl ProcessContext<Self>,
    ) -> Option<PrecisePattern> {
        if let Some(cmds) = self.commands_rx.as_mut() {
            match cmds.pop() {
                Ok(Command::PatternStart(pattern)) => self.set_current_pattern(context, pattern),
                // TODO: handle this command
                Ok(Command::PatternStop(_)) => {
                    self.precise_patterns.get(&String::from("current")).cloned()
                }
                // TODO: handle this command
                Ok(Command::PatternList(_)) => {
                    self.precise_patterns.get(&String::from("current")).cloned()
                }
                // Return the current pattern by default
                Err(PopError::Empty) => {
                    self.precise_patterns.get(&String::from("current")).cloned()
                }
            }
        } else {
            None
        }
    }

    fn set_current_pattern(
        &mut self,
        context: &mut impl ProcessContext<Self>,
        pattern: Pattern,
    ) -> Option<PrecisePattern> {
        let transport = context.transport();

        let precise_pattern = PrecisePattern::from(
            pattern,
            transport.sample_rate,
            transport.tempo.unwrap_or(120.0),
        );
        self.precise_patterns
            .insert(String::from("current"), precise_pattern.clone());

        nih_log!("set current pattern");

        Some(precise_pattern)
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

    fn send(&mut self, context: &mut impl ProcessContext<Self>, sev: SimpleNoteEvent) -> () {
        context.send_event(match sev.note_type {
            NoteType::On => NoteEvent::NoteOn {
                timing: sev.timing,
                voice_id: sev.voice_id,
                channel: sev.channel,
                note: sev.note,
                velocity: sev.velocity,
            },
            NoteType::Off => NoteEvent::NoteOff {
                timing: sev.timing,
                voice_id: sev.voice_id,
                channel: sev.channel,
                note: sev.note,
                velocity: 0.0,
            },
        })
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
            precise_patterns: HashMap::new(),
            commands_rx: None,
            responses_tx: None,
            shutdown_tx: None,
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
    PatternStart(Pattern),
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
    Json(pattern): Json<Pattern>,
) -> response::Result<String, StatusCode> {
    // macro does nothing in the http thread
    // nih_log!("starting pattern {:?}", pattern);
    let mut cmds = controller.commands.lock().unwrap();
    // TODO: handle when the queue is full
    match cmds.push(Command::PatternStart(pattern)) {
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
        let (commands_tx, commands_rx) = RingBuffer::<Command>::new(16); // Arbitrary buffer size
        let (responses_tx, responses_rx) = RingBuffer::<Command>::new(16); // Arbitrary buffer size
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
                .route("/start", post(start_pattern))
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

        if transport.playing {
            if !self.playing {
                self.start(buffer, context);
            } else {
                self.play(buffer, context);
            }
        } else {
            self.playing = false;
        }
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
                    note: Note {
                        note_num: 60,
                        velocity: 0.8,
                        dur_ms: 20,
                    },
                    dur_beats: 1.0,
                },
                Event {
                    note: Note {
                        note_num: 96,
                        velocity: 0.8,
                        dur_ms: 20,
                    },
                    dur_beats: 1.0,
                },
            ],
        };
        let sample_rate = 48000 as f32;
        let tempo = 110 as f64;
        let precise_pattern = PrecisePattern::from(pattern.clone(), sample_rate, tempo);
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
