use crate::controller::{create_router, Command, Controller};
use crate::dur::SongOffsetSamples;
use crate::http_commands::HTTP_LISTEN_PORT;
use crate::pattern::{NamedPattern, Pattern};
use crate::precise::{NoteType, PreciseEventType, PrecisePattern, SimpleNoteEvent};
use nih_plug::prelude::*;
use rtrb::{Consumer, PopError, RingBuffer};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::oneshot;

#[derive(Params)]
pub struct CodeParams {}

impl Default for CodeParams {
    fn default() -> Self {
        Self {}
    }
}

pub struct Code {
    // Plugin thread and command thread will communicate using these.
    pub commands_rx: Option<Consumer<Command>>,
    pub shutdown_tx: Option<oneshot::Sender<()>>,
    pub params: Arc<CodeParams>,
    pub playing: bool,

    patterns: HashMap<String, Pattern>,
    precise_patterns: HashMap<String, PrecisePattern>,
    future_events: HashMap<SongOffsetSamples, Vec<PreciseEventType>>,

    // Command thread will be shutdown by the plugin thread using this.
    tempo_prev_cycle: f64,
}

impl Default for Code {
    fn default() -> Self {
        Self {
            params: Arc::new(CodeParams::default()),
            playing: false,
            // We need the tempo and sample rate to properly initialize this.
            // Will be done on the first process() call.
            patterns: HashMap::new(),
            precise_patterns: HashMap::new(),
            future_events: HashMap::new(),
            commands_rx: None,
            shutdown_tx: None,
            tempo_prev_cycle: 0.0 as f64,
        }
    }
}

pub trait Context<P: Plugin> {
    fn playing(&self) -> bool;
    fn pos_samples(&self) -> Option<i64>;
    fn sample_rate(&self) -> f32;
    fn tempo(&self) -> f64;
    fn send_event(&mut self, event: PluginNoteEvent<P>);
}

impl<P: Plugin, T: ProcessContext<P>> Context<P> for T {
    fn playing(&self) -> bool {
        self.transport().playing
    }

    fn pos_samples(&self) -> Option<i64> {
        self.transport().pos_samples()
    }

    fn sample_rate(&self) -> f32 {
        self.transport().sample_rate
    }

    fn tempo(&self) -> f64 {
        self.transport().tempo.unwrap_or(120.0 as f64)
    }

    fn send_event(&mut self, event: PluginNoteEvent<P>) {
        ProcessContext::send_event(self, event)
    }
}

impl Plugin for Code {
    const NAME: &'static str = "CODE";
    const VENDOR: &'static str = "Brian Sorahan";
    const URL: &'static str = "https://crispy.club";
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
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let commands = Arc::new(Controller {
            commands_tx: Mutex::new(commands_tx),
        });
        self.shutdown_tx = Some(shutdown_tx);
        self.commands_rx = Some(commands_rx);

        thread::spawn(move || {
            let router = create_router(commands);

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let listener =
                    tokio::net::TcpListener::bind(format!("127.0.0.1:{}", HTTP_LISTEN_PORT))
                        .await
                        .unwrap();
                axum::serve(listener, router)
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
        let buf_size = buffer.samples();
        self.cycle(buf_size, context)
    }

    fn deactivate(&mut self) -> () {
        nih_log!("shutting down http thread...");
        if let Some(sender) = self.shutdown_tx.take() {
            sender.send(()).expect("Failed to send shutdown signal");
        }
    }
}

impl Code {
    fn cycle(&mut self, buf_size: usize, ctx: &mut impl Context<Self>) -> ProcessStatus {
        let tempo = ctx.tempo();

        if ctx.playing() {
            if !self.playing {
                self.start(buf_size, ctx);
            } else {
                self.play(buf_size, ctx);
            }
        } else {
            if self.playing {
                self.stop(ctx);
            }
        }
        self.tempo_prev_cycle = tempo;

        ProcessStatus::Normal
    }

    fn start(&mut self, buf_size: usize, ctx: &mut impl Context<Self>) -> ProcessStatus {
        self.playing = true;
        nih_log!(
            "starting CODE (sample rate {}) (tempo {})",
            ctx.sample_rate(),
            ctx.tempo(),
        );
        self.play(buf_size, ctx)
    }

    fn play(&mut self, buf_size: usize, ctx: &mut impl Context<Self>) -> ProcessStatus {
        let pos_samples = ctx.pos_samples().unwrap_or(0) as usize;
        if let Err(_) = self.process_commands(ctx) {
            return ProcessStatus::Error("error processing commands");
        }
        let tempo = ctx.tempo();

        if tempo != self.tempo_prev_cycle {
            nih_log!("recomputing patterns after tempo change");
            self.recompute_patterns(ctx);
        }
        for event in self.get_events(pos_samples, buf_size) {
            if let PreciseEventType::Note(note) = event {
                match note.note_type {
                    NoteType::On => {
                        self.schedule_note_off(note, pos_samples);
                    }
                    _ => {}
                }
            }
            self.send(ctx, event);
        }
        for (song_pos_samples, events) in self.get_future_events(pos_samples, buf_size) {
            for event in events {
                self.send(ctx, event);
            }
            self.future_events.remove(&song_pos_samples);
        }
        ProcessStatus::Normal
    }

    fn stop(&mut self, ctx: &mut impl Context<Self>) -> ProcessStatus {
        self.playing = false;
        nih_log!("turning all notes off");
        self.turn_all_notes_off(ctx);
        ProcessStatus::Normal
    }

    fn schedule_note_off(&mut self, note: SimpleNoteEvent, song_pos_samples: usize) {
        let offset = song_pos_samples + note.note_length_samples;
        if let Some(events) = self.future_events.get_mut(&offset) {
            events.push(PreciseEventType::Note(SimpleNoteEvent {
                note_type: NoteType::Off,
                timing: 0, // TBD when we actually send the note-off
                voice_id: note.voice_id,
                channel: note.channel,
                note: note.note,
                velocity: 0.0,
                note_length_samples: 0 as usize,
            }));
        } else {
            self.future_events.insert(
                offset,
                vec![PreciseEventType::Note(SimpleNoteEvent {
                    note_type: NoteType::Off,
                    timing: 0, // TBD when we actually send the note-off
                    voice_id: note.voice_id,
                    channel: note.channel,
                    note: note.note,
                    velocity: 0.0,
                    note_length_samples: 0 as usize,
                })],
            );
        }
    }

    fn get_future_events(
        &mut self,
        pos_samples: usize,
        buf_size: usize,
    ) -> Vec<(usize, Vec<PreciseEventType>)> {
        self.future_events
            .iter()
            .filter(|pair| {
                let (song_sample_position, _) = pair;
                **song_sample_position >= pos_samples
                    && **song_sample_position > pos_samples + buf_size
            })
            .map(|pair| {
                let (song_sample_position, events) = pair;
                (*song_sample_position, events.clone())
            })
            .collect()
    }

    fn get_events(&mut self, pos_samples: usize, buf_size: usize) -> Vec<PreciseEventType> {
        return self
            .precise_patterns
            .values_mut()
            .into_iter()
            .map(move |precise_pattern| {
                precise_pattern
                    .get_events(pos_samples, pos_samples + buf_size)
                    .into_iter()
            })
            .flatten()
            .collect();
    }

    fn recompute_patterns(&mut self, ctx: &mut impl Context<Self>) {
        for (name, pattern) in self.patterns.iter() {
            let mut playing = false;
            if let Some(existing_pattern) = self.precise_patterns.get(name) {
                playing = existing_pattern.playing;
            }
            nih_log!("recomputing pattern {}", name);
            let precise_pattern = PrecisePattern::from(
                &mut pattern.clone(),
                ctx.sample_rate(),
                ctx.tempo(),
                playing,
            );
            self.precise_patterns
                .insert(name.clone(), precise_pattern.clone());
        }
    }

    fn process_commands(&mut self, ctx: &mut impl Context<Self>) -> Result<(), Box<dyn Error>> {
        if let Some(cmds) = self.commands_rx.as_mut() {
            match cmds.pop() {
                Ok(Command::PatternStart(pattern)) => self.start_pattern(ctx, pattern),
                Ok(Command::PatternStop(name)) => self.stop_pattern(ctx, &name),
                Ok(Command::PatternStopAll) => {
                    for (name, precp) in self.precise_patterns.iter_mut() {
                        nih_log!("stopping pattern {}", name);
                        precp.stop();
                    }
                    Ok(())
                }
                Ok(Command::PatternClear(name)) => {
                    self.precise_patterns.remove(&name);
                    Ok(())
                }
                Ok(Command::PatternClearAll) => {
                    self.precise_patterns.drain();
                    Ok(())
                }
                Err(PopError::Empty) => Ok(()),
            }
        } else {
            Ok(()) // Maybe panic here? This should be unreachable.
        }
    }

    fn start_pattern(
        &mut self,
        ctx: &mut impl Context<Self>,
        named_pattern: NamedPattern,
    ) -> Result<(), Box<dyn Error>> {
        let pattern_length = named_pattern.length_bars;
        nih_log!("starting pattern {}", named_pattern.name);
        let precise_pattern = PrecisePattern::from(
            &mut Pattern {
                channel: named_pattern.channel,
                length_bars: pattern_length,
                events: named_pattern.events.clone(),
            },
            ctx.sample_rate(),
            ctx.tempo(),
            true,
        );
        self.patterns.insert(
            named_pattern.name.clone(),
            Pattern {
                channel: named_pattern.channel,
                length_bars: pattern_length,
                events: named_pattern.events.clone(),
            },
        );
        self.precise_patterns
            .insert(named_pattern.name.clone(), precise_pattern.clone());
        nih_log!(
            "started pattern {} on channel {}",
            named_pattern.name,
            named_pattern.channel
        );
        Ok(())
    }

    fn stop_pattern(
        &mut self,
        ctx: &mut impl Context<Self>,
        name: &str,
    ) -> Result<(), Box<dyn Error>> {
        nih_log!("stopping pattern");
        match self.precise_patterns.get(name) {
            Some(precise_pattern) => {
                let mut clone = precise_pattern.clone();
                clone.playing = false;
                let prev_pattern = self.precise_patterns.insert(String::from(name), clone);
                if let Some(mut pattern) = prev_pattern {
                    let notes_playing = pattern.get_notes_playing();
                    for nev in notes_playing {
                        self.send(ctx, PreciseEventType::Note(nev));
                    }
                }
                nih_log!("stopped pattern {}", name);
                Ok(())
            }
            None => {
                nih_log!("no pattern with name {}", name);
                Ok(())
            }
        }
    }

    fn send(&self, ctx: &mut impl Context<Self>, pevt: PreciseEventType) -> () {
        match pevt {
            PreciseEventType::Note(nev) => match nev.note_type {
                NoteType::On => {
                    // nih_log!("note {} played on channel {}", nev.note, nev.channel - 1);
                    ctx.send_event(NoteEvent::NoteOn {
                        timing: nev.timing,
                        voice_id: nev.voice_id,
                        channel: nev.channel - 1,
                        note: nev.note,
                        velocity: nev.velocity,
                    })
                }
                NoteType::Off => ctx.send_event(NoteEvent::NoteOff {
                    timing: nev.timing,
                    voice_id: nev.voice_id,
                    channel: nev.channel - 1,
                    note: nev.note,
                    velocity: 0.0,
                }),
                NoteType::Rest => {}
            },
            PreciseEventType::Ctrl(cev) => {
                ctx.send_event(NoteEvent::MidiCC {
                    timing: cev.timing,
                    channel: cev.channel - 1,
                    cc: cev.cc,
                    value: cev.value,
                });
            }
            PreciseEventType::VoiceTerminated(vt) => {
                ctx.send_event(NoteEvent::VoiceTerminated {
                    timing: vt.timing,
                    channel: vt.channel - 1,
                    voice_id: vt.voice_id,
                    note: vt.note,
                });
            }
        }
    }

    fn turn_all_notes_off(&mut self, ctx: &mut impl Context<Self>) -> () {
        for event in self
            .precise_patterns
            .values_mut()
            .into_iter()
            .map(move |ppat| ppat.get_notes_playing().into_iter())
            .flatten()
            .collect::<Vec<SimpleNoteEvent>>()
        {
            self.send(ctx, PreciseEventType::Note(event));
        }
    }
}

impl ClapPlugin for Code {
    const CLAP_ID: &'static str = "club.crispy";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("MIDI sequencing via code that is edited on the fly");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect, ClapFeature::Utility];
}

impl Vst3Plugin for Code {
    const VST3_CLASS_ID: [u8; 16] = *b"XXXXXXXXXXXXXXXX";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Tools];
}

nih_export_clap!(Code);
nih_export_vst3!(Code);

#[cfg(test)]
mod tests {
    use crate::plugin::Code;
    use nih_plug::prelude::*;

    struct FakeInitContext;

    impl<P: Plugin> InitContext<P> for FakeInitContext {
        fn plugin_api(&self) -> PluginApi {
            PluginApi::Clap
        }
        fn execute(&self, _task: P::BackgroundTask) {}
        fn set_latency_samples(&self, _samples: u32) {}
        fn set_current_voice_capacity(&self, _capacity: u32) {}
    }

    #[test]
    fn test_plugin_initialize() -> Result<(), String> {
        let mut plugin: Code = Default::default();
        assert!(plugin.initialize(
            &AudioIOLayout::default(),
            &BufferConfig {
                sample_rate: 48000 as f32,
                min_buffer_size: Some(256),
                max_buffer_size: 4096,
                process_mode: ProcessMode::Realtime,
            },
            &mut FakeInitContext {},
        ));
        Ok(())
    }
}
