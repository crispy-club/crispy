use crispy_code::controller::Command;
use crispy_code::dsl::notes;
use crispy_code::plugin::Code;
use crispy_code::plugin_export::Context;
use crispy_code::precise::{NoteType, PreciseEventType, SimpleNoteEvent, VoiceTerminatedEvent};
use nih_plug::prelude::*;
use std::collections::HashMap;

struct FakeInitContext;

impl<P: Plugin> InitContext<P> for FakeInitContext {
    fn plugin_api(&self) -> PluginApi {
        PluginApi::Clap
    }
    fn execute(&self, _task: P::BackgroundTask) {}
    fn set_latency_samples(&self, _samples: u32) {}
    fn set_current_voice_capacity(&self, _capacity: u32) {}
}

type BufNum = usize;

struct PluginTest {
    tests: HashMap<BufNum, CycleTest>,
    buf_size: usize,
    sample_rate: f32,
    tempo: f64,
}

impl PluginTest {
    fn run(&self) -> Result<(), String> {
        let mut plugin = Code::default();
        let controller = plugin.tests_init();
        let mut cmds = controller.commands_tx.lock().unwrap();
        let max_buf_num = *self.tests.keys().max().unwrap();
        let mut song_pos_samples: i64 = 0;
        for buf_num in 0..(max_buf_num + 1) {
            let cycle_test_opt = self.tests.get(&buf_num);
            if let Some(cycle_test) = cycle_test_opt {
                for cmd in &cycle_test.commands {
                    assert!(cmds.push(cmd.clone()).is_ok());
                }
            }
            let ctx = Context {
                playing: true,
                pos_samples: song_pos_samples,
                sample_rate: self.sample_rate,
                tempo: self.tempo,
            };
            let (status, events) = plugin.cycle(self.buf_size, &ctx);
            if cycle_test_opt.is_none() {
                // If there was no test specified for this buf_num then
                // the plugin shouldn't have generated any events.
                // This forces tests to specify _all_ the events the plugin
                // will generate.
                assert_eq!(events.len(), 0);
                assert_eq!(status, ProcessStatus::Normal);
                song_pos_samples += self.buf_size as i64;
                continue;
            }
            let cycle_test = cycle_test_opt.unwrap();
            assert_eq!(status, cycle_test.exp_status);
            assert_eq!(
                cycle_test.exp_events.len(),
                events.len(),
                "buf_num {:?} events {:?}",
                buf_num,
                events
            );
            for (idx, event) in events.iter().enumerate() {
                assert_eq!(
                    *event, cycle_test.exp_events[idx],
                    "buf_num {:?} event {:?}",
                    buf_num, *event,
                );
            }
            song_pos_samples += self.buf_size as i64;
        }
        plugin.deactivate();
        Ok(())
    }
}

// Single invocation of the plugin's cycle method.
// Sends commands to the plugin, calls cycle(), verifies events were output.
struct CycleTest {
    commands: Vec<Command>,
    exp_events: Vec<PreciseEventType>,
    exp_status: ProcessStatus,
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
    let p1 = notes("Cx D'g").map(|p| p.named("foo")).unwrap();
    let _p2 = notes("Cx D'g Gm").map(|p| p.named("foo")).unwrap();

    let test = PluginTest {
        tests: HashMap::from([
            (
                0,
                CycleTest {
                    commands: vec![Command::PatternStart(p1)],
                    exp_events: vec![PreciseEventType::Note(SimpleNoteEvent {
                        note_type: NoteType::On,
                        timing: 0 as u32,
                        voice_id: Some(0),
                        channel: 1,
                        note: 60,
                        velocity: 0.89,
                        note_length_samples: 24000 as usize,
                    })],
                    exp_status: ProcessStatus::Normal,
                },
            ),
            (
                93,
                CycleTest {
                    commands: vec![],
                    exp_events: vec![
                        PreciseEventType::Note(SimpleNoteEvent {
                            note_type: NoteType::Off,
                            timing: 192 as u32,
                            voice_id: Some(0),
                            channel: 1,
                            note: 60,
                            velocity: 0.0,
                            note_length_samples: 0 as usize,
                        }),
                        PreciseEventType::VoiceTerminated(VoiceTerminatedEvent {
                            timing: 192,
                            channel: 1,
                            voice_id: Some(0),
                            note: 60,
                        }),
                    ],
                    exp_status: ProcessStatus::Normal,
                },
            ),
        ]),
        buf_size: 256 as usize,
        sample_rate: 48000.0,
        tempo: 120.0,
    };
    test.run()
}
