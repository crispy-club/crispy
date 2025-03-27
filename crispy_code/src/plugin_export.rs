use crate::controller::{create_router, Command, Controller};
use crate::http_commands::HTTP_LISTEN_PORT;
use crate::plugin::Code;
use nih_plug::prelude::*;
use rtrb::RingBuffer;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::oneshot;

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
