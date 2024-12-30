use crate::controller::{start_pattern, stop_pattern, Command, Controller};
use crate::pattern::{
    FractionalDuration, NamedPattern, NoteType, Pattern, PrecisePattern, SimpleNoteEvent,
};
use axum::{routing::post, Router};
use nih_plug::prelude::*;
use rtrb::{Consumer, PopError, Producer, RingBuffer};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::oneshot;

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
        let mut precise_patterns = self.precise_patterns.clone();

        for precise_pattern in precise_patterns.values_mut() {
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
                &mut pattern.clone(),
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
        let pattern_length = named_pattern
            .length_bars
            .or(Some(FractionalDuration { num: 1, den: 1 }));
        let precise_pattern = PrecisePattern::from(
            &mut Pattern {
                length_bars: pattern_length,
                events: named_pattern.events.clone(),
            },
            transport.sample_rate,
            transport.tempo.unwrap_or(120.0),
            true,
        );
        self.patterns.insert(
            named_pattern.name.clone(),
            Pattern {
                length_bars: pattern_length,
                events: named_pattern.events.clone(),
            },
        );
        let prev_pattern = self
            .precise_patterns
            .insert(named_pattern.name.clone(), precise_pattern.clone());
        if let Some(mut pattern) = prev_pattern {
            let notes_playing = pattern.get_notes_playing();
            notes_playing.into_iter().for_each(|nev| {
                self.send(context, nev);
            });
        }
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
