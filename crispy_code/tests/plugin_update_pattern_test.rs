use crispy_code::plugin::Code;
use crispy_code::plugin_export::Context;
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

#[test]
fn test_plugin_pattern_update() -> Result<(), String> {
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
    let ctx = Context {
        playing: true,
        pos_samples: 0,
        sample_rate: 48000.0,
        tempo: 120.0,
    };
    let buf_size = 256 as usize;
    let (status, _events) = plugin.cycle(buf_size, &ctx);
    assert_eq!(status, ProcessStatus::Normal);
    Ok(())
}
