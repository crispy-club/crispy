use crate::controller::{create_router, Command, Controller};
use crate::http_commands::HTTP_LISTEN_PORT;
use crate::plugin::Code;
use crate::precise::{NoteType, PreciseEventType};
use nih_plug::prelude::*;
use rtrb::RingBuffer;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::oneshot;

pub struct Context {
    pub playing: bool,
    pub pos_samples: i64,
    pub sample_rate: f32,
    pub tempo: f64,
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
        let ctx = Context {
            playing: context.transport().playing,
            pos_samples: context.transport().pos_samples().unwrap_or(0), // TODO: track pos_samples ourselves
            sample_rate: context.transport().sample_rate,
            tempo: context.transport().tempo.unwrap_or(120.),
        };
        let (process_status, events) = self.cycle(buf_size, &ctx);
        if matches!(process_status, ProcessStatus::Error(_)) {
            return process_status;
        }
        for event in events {
            let nih_event = to_nih_event::<Self>(event);
            if nih_event.is_none() {
                continue;
            }
            context.send_event(nih_event.unwrap());
        }
        ProcessStatus::Normal
    }

    fn deactivate(&mut self) -> () {
        nih_log!("shutting down http thread...");
        if let Some(sender) = self.shutdown_tx.take() {
            sender.send(()).expect("Failed to send shutdown signal");
        }
    }
}

fn to_nih_event<P: Plugin>(pevt: PreciseEventType) -> Option<PluginNoteEvent<P>> {
    match pevt {
        PreciseEventType::Note(nev) => match nev.note_type {
            NoteType::On => Some(NoteEvent::NoteOn {
                timing: nev.timing,
                voice_id: nev.voice_id,
                channel: nev.channel - 1,
                note: nev.note,
                velocity: nev.velocity,
            }),
            NoteType::Off => Some(NoteEvent::NoteOff {
                timing: nev.timing,
                voice_id: nev.voice_id,
                channel: nev.channel - 1,
                note: nev.note,
                velocity: 0.0,
            }),
            NoteType::Rest => None,
        },
        PreciseEventType::Ctrl(cev) => Some(NoteEvent::MidiCC {
            timing: cev.timing,
            channel: cev.channel - 1,
            cc: cev.cc,
            value: cev.value,
        }),
        PreciseEventType::VoiceTerminated(vt) => Some(NoteEvent::VoiceTerminated {
            timing: vt.timing,
            channel: vt.channel - 1,
            voice_id: vt.voice_id,
            note: vt.note,
        }),
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
