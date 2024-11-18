use nih_plug::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct Note {
    note_num: u8,
    dur_ms: i64,
    velocity: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Event {
    note: Note,
    dur_beats: f64,
}

#[derive(Clone, Debug)]
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
            nih_log!(
                "added note on event at sample {}, note off event at {}",
                sample_idx,
                note_off_timing,
            );
            sample_idx += ((samples_per_beat as f64) * event.dur_beats) as usize;
        }
        return PrecisePattern {
            events: events_map,
            length_samples: sample_idx,
        };
    }

    fn get_events(&self, start: usize, end: usize) -> Vec<SimpleNoteEvent> {
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

pub struct WasmSeq {
    params: Arc<WasmSeqParams>,
    pattern: Pattern,
    playing: bool,
    precise_pattern: Option<PrecisePattern>,
}

impl WasmSeq {
    fn play(
        &mut self,
        buffer: &mut Buffer,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let transport = context.transport();
        let pos_samples = transport.pos_samples().unwrap_or(0) as usize;

        if let Some(precise_pattern) = &self.precise_pattern {
            for event in precise_pattern.get_events(pos_samples, pos_samples + buffer.samples()) {
                nih_log!(
                    "playing event {:?} within buffer starting at {} and ending at {}",
                    event,
                    pos_samples,
                    pos_samples + buffer.samples()
                );
                self.send(context, event);
            }
        }
        ProcessStatus::Normal
    }

    fn start(
        &mut self,
        buffer: &mut Buffer,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let transport = context.transport();

        let precise_pattern = PrecisePattern::from(
            self.pattern.clone(),
            transport.sample_rate,
            transport.tempo.unwrap_or(120.0),
        );
        self.playing = true;

        nih_log!(
            "starting wasm seq (sample rate {}) (tempo {}) (default pattern length {} samples)",
            transport.sample_rate,
            transport.tempo.unwrap_or(120.0),
            precise_pattern.length_samples,
        );
        self.precise_pattern = Some(precise_pattern);
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
struct WasmSeqParams {
    #[id = "note"]
    pub note: IntParam,

    #[id = "dur_ms"]
    pub dur_ms: IntParam,
}

impl Default for WasmSeq {
    fn default() -> Self {
        Self {
            params: Arc::new(WasmSeqParams::default()),
            playing: false,
            pattern: Pattern {
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
            },
            // We need the tempo and sample rate to properly initialize this.
            // Will be done on the first process() call.
            precise_pattern: None,
        }
    }
}

impl Default for WasmSeqParams {
    fn default() -> Self {
        Self {
            note: IntParam::new("MIDI Note", 60, IntRange::Linear { min: 0, max: 127 }),
            dur_ms: IntParam::new(
                "Note Duration (ms)",
                20,
                IntRange::Linear { min: 5, max: 200 },
            ),
        }
    }
}

impl Plugin for WasmSeq {
    const NAME: &'static str = "WASM Seq";
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
        nih_log!("deactivating...");
    }
}

impl ClapPlugin for WasmSeq {
    const CLAP_ID: &'static str = "net.sorahan.brian.wasmseq";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Trigger a midi note every beat");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect, ClapFeature::Utility];
}

impl Vst3Plugin for WasmSeq {
    const VST3_CLASS_ID: [u8; 16] = *b"XXXXXXXXXXXXXXXX";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Tools];
}

nih_export_clap!(WasmSeq);
nih_export_vst3!(WasmSeq);

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
