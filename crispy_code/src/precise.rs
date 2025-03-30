use crate::dur::{PatternOffsetSamples, SongOffsetSamples};
use crate::pattern::{CtrlEvent, Event, EventType, Note, Pattern};
use nih_plug::nih_log;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum NoteType {
    On,
    Off,
    Rest,
}

type Channel = u8;
type NoteNum = u8;
type VoiceID = i32;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SimpleNoteEvent {
    pub note_type: NoteType,
    pub timing: u32,
    pub voice_id: Option<VoiceID>,
    pub channel: Channel,
    pub note: u8,
    pub velocity: f32,
    pub note_length_samples: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SimpleCtrlEvent {
    pub timing: u32,
    pub channel: Channel,
    pub cc: u8,
    pub value: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct VoiceTerminatedEvent {
    pub timing: u32,
    pub channel: Channel,
    pub voice_id: Option<VoiceID>,
    pub note: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum PreciseEventType {
    Note(SimpleNoteEvent),
    Ctrl(SimpleCtrlEvent),
    VoiceTerminated(VoiceTerminatedEvent),
}

#[derive(Clone, PartialEq, Serialize)]
pub struct PrecisePattern {
    pub events: HashMap<PatternOffsetSamples, Vec<PreciseEventType>>,
    pub length_samples: usize,
    pub playing: bool,
    // (channel, note) -> voice_id
    pub notes_playing: HashMap<(Channel, NoteNum), i32>,

    future_events: HashMap<SongOffsetSamples, Vec<PreciseEventType>>,
}

pub fn compute_extra_samples(samples_remainder: i64, num_events: usize) -> Vec<i64> {
    let samps_per_event = samples_remainder / (num_events as i64);
    let mut remainder = samples_remainder % (num_events as i64);
    let mut extra_samples = vec![samps_per_event; num_events];
    for idx in 0..num_events {
        if remainder > 0 {
            extra_samples[idx] = samps_per_event + 1;
        } else {
            extra_samples[idx] = samps_per_event;
        }
        remainder -= 1;
    }
    extra_samples
}

fn insert_note(
    events_map: &mut HashMap<usize, Vec<PreciseEventType>>,
    event: &Event,
    note: &Note,
    channel: u8,
    tick_length_samples: i64,
    _pattern_length_samples: usize,
    sample_idx: usize,
) {
    let event_length_samples = event.dur.num * tick_length_samples;
    let note_length_samples = (note.dur.num * event_length_samples) / note.dur.den;
    events_map.insert(
        sample_idx,
        vec![PreciseEventType::Note(SimpleNoteEvent {
            note_type: NoteType::On,
            timing: sample_idx as u32,
            voice_id: None,
            channel: channel,
            note: note.note_num,
            velocity: note.velocity,
            note_length_samples: note_length_samples as usize,
        })],
    );
}

fn insert_ctrl(
    events_map: &mut HashMap<usize, Vec<PreciseEventType>>,
    ctrl: &CtrlEvent,
    channel: u8,
    sample_idx: usize,
) {
    events_map.insert(
        sample_idx,
        vec![PreciseEventType::Ctrl(SimpleCtrlEvent {
            timing: sample_idx as u32,
            channel: channel,
            cc: ctrl.cc,
            value: ctrl.value,
        })],
    );
}

fn insert_event(
    events_map: &mut HashMap<usize, Vec<PreciseEventType>>,
    event: &Event,
    channel: u8,
    tick_length_samples: i64,
    pattern_length_samples: usize,
    sample_idx: usize,
) {
    match &event.action {
        EventType::MultiNoteEvent(notes) => {
            for note in notes {
                insert_note(
                    events_map,
                    event,
                    &note,
                    channel,
                    tick_length_samples,
                    pattern_length_samples,
                    sample_idx,
                );
            }
        }
        EventType::NoteEvent(note) => {
            insert_note(
                events_map,
                event,
                &note,
                channel,
                tick_length_samples,
                pattern_length_samples,
                sample_idx,
            );
        }
        EventType::Rest => {
            events_map.insert(
                sample_idx,
                vec![PreciseEventType::Note(SimpleNoteEvent {
                    note_type: NoteType::Rest,
                    timing: sample_idx as u32,
                    voice_id: None,
                    channel: 1,
                    note: 0,
                    velocity: 0.0,
                    note_length_samples: 0 as usize, // FIXME
                })],
            );
        }
        EventType::Ctrl(ctrl) => {
            insert_ctrl(events_map, &ctrl, channel, sample_idx);
        }
    }
}

impl PrecisePattern {
    pub fn start(&mut self) {
        self.playing = false
    }

    pub fn stop(&mut self) {
        self.playing = false
    }

    pub fn from(
        pattern: &mut Pattern,
        sample_rate: f32,
        tempo: f64,
        playing: bool,
    ) -> PrecisePattern {
        let samples_per_bar = (sample_rate as f64 * (240.0 / tempo)) as i64;
        if pattern.events.len() == 0 {
            // I added this to try to track down a potential bug in the plugin where
            // it unexpectedly is trying to initialize empty patterns...
            nih_log!("creating empty pattern that will not play... intentional?");
            return PrecisePattern {
                events: HashMap::new(),
                length_samples: samples_per_bar as usize,
                playing: false,
                notes_playing: HashMap::new(),
                future_events: HashMap::new(),
            };
        }
        let pattern_length_samples =
            (pattern.length_bars.num * samples_per_bar) / pattern.length_bars.den;
        let least_common_multiple = pattern.compute_events_lcm();
        let tick_length_samples = pattern_length_samples / least_common_multiple;
        let samples_remainder = pattern_length_samples % least_common_multiple;
        let extra_samples = compute_extra_samples(samples_remainder, pattern.events.len());

        let pattern_length_samples: usize = pattern
            .events
            .clone()
            .into_iter()
            .enumerate()
            .map(|pair| ((tick_length_samples * pair.1.dur.num) + extra_samples[pair.0]) as usize)
            .sum();

        let mut sample_idx: usize = 0;
        let mut events_map: HashMap<usize, Vec<PreciseEventType>> = HashMap::new();

        for (idx, event) in pattern.events.clone().into_iter().enumerate() {
            insert_event(
                &mut events_map,
                &event,
                pattern.channel,
                tick_length_samples,
                pattern_length_samples,
                sample_idx,
            );
            sample_idx += ((tick_length_samples * event.dur.num) + extra_samples[idx]) as usize;
        }
        return PrecisePattern {
            events: events_map,
            length_samples: pattern_length_samples,
            playing: playing,
            notes_playing: HashMap::new(),
            future_events: HashMap::new(),
        };
    }

    pub fn get_events(&mut self, pos_samples: i64, buf_size: usize) -> Vec<PreciseEventType> {
        let end = (pos_samples as usize) + buf_size;
        let mut events = self.get_curr_events(pos_samples, end);
        // Schedule note-off events for any note-on events output during the current cycle.
        for event in &events {
            if let PreciseEventType::Note(note) = *event {
                match note.note_type {
                    NoteType::On => {
                        self.schedule_note_off(note, pos_samples, end);
                    }
                    _ => {}
                }
            }
        }
        // Play any events that were scheduled in the future.
        for (event_song_pos_samples, fut_events) in self.get_future_events(pos_samples, buf_size) {
            for event in fut_events {
                println!(
                    "added a future event {:?} because its sample offset {} between {} and {}",
                    event,
                    event_song_pos_samples,
                    pos_samples,
                    pos_samples + (buf_size as i64),
                );
                events.push(event);
            }
            self.future_events.remove(&event_song_pos_samples);
        }
        events
    }

    pub fn get_curr_events(&mut self, pos_samples: i64, end: usize) -> Vec<PreciseEventType> {
        if self.length_samples == 0 || !self.playing {
            return vec![];
        }
        let adj_start = (pos_samples as usize) % self.length_samples;
        let adj_end = end % self.length_samples;

        if adj_end < adj_start {
            let mut pat_end = self.get_events_adj(adj_start, self.length_samples, 0);
            let next_pat = self.get_events_adj(0, adj_end, self.length_samples - adj_start);
            pat_end.extend(next_pat);
            return pat_end;
        }
        return self.get_events_adj(adj_start, adj_end, 0);
    }

    pub fn get_events_adj(
        &mut self,
        adj_start: usize,
        adj_end: usize,
        timing_offset: usize,
    ) -> Vec<PreciseEventType> {
        let mut selected_events: Vec<PreciseEventType> = Vec::new();

        for sample_index in adj_start..adj_end {
            match self.events.get(&sample_index) {
                None => (),
                // Need to adjust the timing so that it is relative to the current audio buffer.
                Some(events) => events.into_iter().for_each(|pevt| match pevt {
                    PreciseEventType::Note(nev) => {
                        let timing = ((nev.timing as usize) - adj_start + timing_offset) as u32;
                        match nev.note_type {
                            NoteType::Rest => {}
                            NoteType::Off => {
                                let voice_id = self.notes_playing.remove(&(nev.channel, nev.note));

                                if let Some(vid) = voice_id {
                                    selected_events.push(PreciseEventType::Note(SimpleNoteEvent {
                                        note_type: nev.note_type,
                                        timing: timing,
                                        voice_id: voice_id,
                                        channel: nev.channel,
                                        note: nev.note,
                                        velocity: nev.velocity,
                                        note_length_samples: nev.note_length_samples,
                                    }));
                                    selected_events.push(PreciseEventType::VoiceTerminated(
                                        VoiceTerminatedEvent {
                                            timing: timing,
                                            voice_id: Some(vid),
                                            channel: nev.channel,
                                            note: nev.note,
                                        },
                                    ));
                                }
                            }
                            NoteType::On => {
                                let new_voice_id = self.notes_playing.len() as i32;
                                self.notes_playing
                                    .insert((nev.channel, nev.note), new_voice_id);
                                selected_events.push(PreciseEventType::Note(SimpleNoteEvent {
                                    note_type: nev.note_type,
                                    timing: timing,
                                    voice_id: Some(new_voice_id),
                                    channel: nev.channel,
                                    note: nev.note,
                                    velocity: nev.velocity,
                                    note_length_samples: nev.note_length_samples,
                                }));
                            }
                        };
                    }
                    PreciseEventType::Ctrl(ev) => {
                        let timing = ((ev.timing as usize) - adj_start + timing_offset) as u32;
                        selected_events.push(PreciseEventType::Ctrl(SimpleCtrlEvent {
                            timing: timing,
                            channel: ev.channel,
                            cc: ev.cc,
                            value: ev.value,
                        }));
                    }
                    PreciseEventType::VoiceTerminated(vt) => {
                        let timing = ((vt.timing as usize) - adj_start + timing_offset) as u32;
                        selected_events.push(PreciseEventType::VoiceTerminated(
                            VoiceTerminatedEvent {
                                timing: timing,
                                voice_id: vt.voice_id,
                                channel: vt.channel,
                                note: vt.note,
                            },
                        ));
                    }
                }),
            }
        }
        selected_events
    }

    // For any notes that are currently on, will return a NoteOff event.
    pub fn get_notes_playing(&mut self) -> Vec<SimpleNoteEvent> {
        let notes_playing = self
            .notes_playing
            .iter()
            .map(|(k, voice_id)| SimpleNoteEvent {
                note_type: NoteType::Off,
                timing: 0,
                voice_id: Some(*voice_id),
                channel: k.0,
                note: k.1,
                velocity: 0.0,
                note_length_samples: 0 as usize, // FIXME
            })
            .collect();
        self.notes_playing = HashMap::new();
        notes_playing
    }

    fn schedule_note_off(&mut self, note_on: SimpleNoteEvent, pos_samples: i64, end: usize) {
        assert!(matches!(note_on.note_type, NoteType::On));
        let buf_size = end - (pos_samples as usize);
        let len = note_on.note_length_samples as u32;
        let norem = (buf_size as u32) - note_on.timing;
        let bs = buf_size as u32;
        let note_off_timing = (len - norem) - (((len - norem) / bs) * bs);
        let offset =
            (pos_samples as usize) + note_on.note_length_samples + (note_on.timing as usize);
        if let Some(events) = self.future_events.get_mut(&offset) {
            events.push(PreciseEventType::Note(SimpleNoteEvent {
                note_type: NoteType::Off,
                timing: note_off_timing,
                voice_id: note_on.voice_id,
                channel: note_on.channel,
                note: note_on.note,
                velocity: 0.0,
                note_length_samples: 0 as usize,
            }));
            events.push(PreciseEventType::VoiceTerminated(VoiceTerminatedEvent {
                timing: note_off_timing,
                voice_id: note_on.voice_id,
                channel: note_on.channel,
                note: note_on.note,
            }));
        } else {
            println!(
                "scheduled note off to happen at sample {} (song_pos_samples = {})",
                offset, pos_samples
            );
            self.future_events.insert(
                offset,
                vec![
                    PreciseEventType::Note(SimpleNoteEvent {
                        note_type: NoteType::Off,
                        timing: note_off_timing as u32,
                        voice_id: note_on.voice_id,
                        channel: note_on.channel,
                        note: note_on.note,
                        velocity: 0.0,
                        note_length_samples: 0 as usize,
                    }),
                    PreciseEventType::VoiceTerminated(VoiceTerminatedEvent {
                        timing: note_off_timing,
                        voice_id: note_on.voice_id,
                        channel: note_on.channel,
                        note: note_on.note,
                    }),
                ],
            );
        }
    }

    fn get_future_events(
        &mut self,
        pos_samples: i64,
        buf_size: usize,
    ) -> Vec<(usize, Vec<PreciseEventType>)> {
        self.future_events
            .iter()
            .filter(|pair| {
                let (song_sample_position, _) = pair;
                **song_sample_position >= (pos_samples as usize)
                    && **song_sample_position < (pos_samples as usize) + buf_size
            })
            .map(|pair| {
                let (song_sample_position, events) = pair;
                (*song_sample_position, events.clone())
            })
            .collect()
    }
}

impl fmt::Debug for PrecisePattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let js = serde_json::to_string(self).map_err(|_| fmt::Error)?;
        f.write_str(&js)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::dur::{Dur, BAR};
    use crate::pattern::{CtrlEvent, Event, EventType, Note, Pattern};
    use crate::precise::{
        compute_extra_samples, NoteType, PreciseEventType, PrecisePattern, SimpleCtrlEvent,
        SimpleNoteEvent, VoiceTerminatedEvent,
    };
    use std::collections::HashMap;

    fn verify_pattern_playback(
        pattern: &Pattern,
        expectations: &HashMap<usize, Vec<PreciseEventType>>,
    ) -> Result<(), String> {
        let sample_rate = 48000 as f32;
        let tempo = 110 as f64;
        let mut precise_pattern =
            PrecisePattern::from(&mut pattern.clone(), sample_rate, tempo, true);
        let buffer_size_samples = 256 as usize;
        let max_buf_num = *expectations.keys().max().unwrap();

        for bufnum in 0..(max_buf_num + 1) {
            let expected_events_opt = expectations.get(&bufnum);
            let buffer_start_samples = bufnum * buffer_size_samples;
            let events =
                precise_pattern.get_events(buffer_start_samples as i64, buffer_size_samples);

            match expected_events_opt {
                Some(expected_events) => {
                    assert_eq!(
                        *expected_events, events,
                        "expected {:?}, got {:?} (bufnum {:?})",
                        expected_events, events, bufnum
                    );
                }
                None => {
                    assert_eq!(
                        events.len(),
                        0,
                        "expected no events, got {:?} (bufnum={:?})",
                        events,
                        bufnum
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_compute_extra_samples() -> Result<(), String> {
        let extra_samples = compute_extra_samples(37, 5);
        assert_eq!(extra_samples, vec![8, 8, 7, 7, 7]);
        Ok(())
    }

    #[test]
    fn test_precise_pattern_empty() {
        let pat = &mut Pattern {
            channel: 1,
            events: vec![],
            length_bars: BAR,
        };
        let sample_rate = 48000 as f32;
        let tempo_bpm = 120 as f64;
        let is_playing = true;
        let ppat = PrecisePattern::from(pat, sample_rate, tempo_bpm, is_playing);
        assert_eq!(
            ppat,
            PrecisePattern {
                events: HashMap::new(),
                length_samples: 96000,
                playing: false, // empty pattern just gets turned off by default
                notes_playing: HashMap::new(),
                future_events: HashMap::new(),
            }
        );
    }

    #[test]
    fn test_precise_pattern_notes() -> Result<(), String> {
        let pattern = Pattern {
            channel: 1,
            length_bars: Dur { num: 1, den: 2 },
            events: vec![
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 60,
                        velocity: 0.8,
                        dur: Dur { num: 1, den: 4 },
                    }),
                    dur: Dur { num: 1, den: 2 },
                },
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 96,
                        velocity: 0.8,
                        dur: Dur { num: 1, den: 4 },
                    }),
                    dur: Dur { num: 1, den: 2 },
                },
            ],
        };
        let expectations: HashMap<usize, Vec<PreciseEventType>> = HashMap::from([
            (
                0,
                vec![PreciseEventType::Note(SimpleNoteEvent {
                    note_type: NoteType::On,
                    timing: 0,
                    voice_id: Some(0),
                    channel: 1,
                    note: 60,
                    velocity: 0.8,
                    note_length_samples: 6545 as usize,
                })],
            ),
            (
                25,
                vec![
                    PreciseEventType::Note(SimpleNoteEvent {
                        note_type: NoteType::Off,
                        timing: 145,
                        voice_id: Some(0),
                        channel: 1,
                        note: 60,
                        velocity: 0.0,
                        note_length_samples: 0 as usize, // FIXME
                    }),
                    PreciseEventType::VoiceTerminated(VoiceTerminatedEvent {
                        timing: 145,
                        voice_id: Some(0),
                        channel: 1,
                        note: 60,
                    }),
                ],
            ),
            (
                102,
                vec![PreciseEventType::Note(SimpleNoteEvent {
                    note_type: NoteType::On,
                    timing: 70,
                    voice_id: Some(1),
                    channel: 1,
                    note: 96,
                    velocity: 0.8,
                    note_length_samples: 6545 as usize,
                })],
            ),
            (
                127,
                vec![
                    PreciseEventType::Note(SimpleNoteEvent {
                        note_type: NoteType::Off,
                        timing: 215,
                        voice_id: Some(1),
                        channel: 1,
                        note: 96,
                        velocity: 0.0,
                        note_length_samples: 0 as usize, // FIXME
                    }),
                    PreciseEventType::VoiceTerminated(VoiceTerminatedEvent {
                        timing: 215,
                        voice_id: Some(1),
                        channel: 1,
                        note: 96,
                    }),
                ],
            ),
            // Add this last one to see how the code behaves when the pattern loops.
            (
                204,
                vec![PreciseEventType::Note(SimpleNoteEvent {
                    note_type: NoteType::On,
                    timing: 139,
                    voice_id: Some(2),
                    channel: 1,
                    note: 60,
                    velocity: 0.8,
                    note_length_samples: 6545 as usize, // FIXME
                })],
            ),
        ]);
        verify_pattern_playback(&pattern, &expectations)
    }

    #[test]
    fn test_precise_pattern_ctrl() -> Result<(), String> {
        let pattern = Pattern {
            channel: 1,
            length_bars: Dur { num: 1, den: 2 },
            events: vec![
                Event {
                    action: EventType::Ctrl(CtrlEvent {
                        cc: 102,
                        value: 0.8,
                    }),
                    dur: Dur { num: 1, den: 2 },
                },
                Event {
                    action: EventType::Ctrl(CtrlEvent {
                        cc: 102,
                        value: 0.4,
                    }),
                    dur: Dur { num: 1, den: 2 },
                },
            ],
        };
        let expectations: HashMap<usize, Vec<PreciseEventType>> = HashMap::from([
            (
                0,
                vec![PreciseEventType::Ctrl(SimpleCtrlEvent {
                    timing: 0,
                    channel: 1,
                    cc: 102,
                    value: 0.8,
                })],
            ),
            (
                102,
                vec![PreciseEventType::Ctrl(SimpleCtrlEvent {
                    timing: 70,
                    channel: 1,
                    cc: 102,
                    value: 0.4,
                })],
            ),
        ]);
        verify_pattern_playback(&pattern, &expectations)
    }

    #[test]
    fn test_precise_pattern_overlapping_notes() -> Result<(), String> {
        let pattern = Pattern {
            channel: 1,
            length_bars: Dur { num: 1, den: 1 },
            events: vec![
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 60,
                        velocity: 0.8,
                        dur: Dur { num: 2, den: 1 },
                    }),
                    dur: Dur { num: 1, den: 2 },
                },
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 96,
                        velocity: 0.8,
                        dur: Dur { num: 2, den: 1 },
                    }),
                    dur: Dur { num: 1, den: 2 },
                },
            ],
        };
        let expectations: HashMap<usize, Vec<PreciseEventType>> = HashMap::from([
            (
                0, // Beginning of the pattern
                vec![PreciseEventType::Note(SimpleNoteEvent {
                    note_type: NoteType::On,
                    timing: 0,
                    voice_id: Some(0),
                    channel: 1,
                    note: 60,
                    velocity: 0.8,
                    note_length_samples: 104726 as usize,
                })],
            ),
            (
                204, // Halfway through the pattern
                vec![
                    // PreciseEventType::Note(SimpleNoteEvent {
                    //     note_type: NoteType::Off,
                    //     timing: 139, // sample pos 52363
                    //     voice_id: None,
                    //     channel: 1,
                    //     note: 96,
                    //     velocity: 0.0,
                    //     note_length_samples: 0 as usize, // FIXME
                    // }),
                    PreciseEventType::Note(SimpleNoteEvent {
                        note_type: NoteType::On,
                        timing: 140, // sample pos 52364
                        voice_id: Some(1),
                        channel: 1,
                        note: 96,
                        velocity: 0.8,
                        note_length_samples: 104726 as usize,
                    }),
                ],
            ),
            (
                409, // End of the pattern (loops back on itself)
                vec![
                    //     Note(SimpleNoteEvent {
                    //         note_type: On,
                    //         timing: 23,
                    //         voice_id: Some(2),
                    //         channel: 1,
                    //         note: 60,
                    //         velocity: 0.8,
                    //         note_length_samples: 104726,
                    //     }),
                    //     Note(SimpleNoteEvent {
                    //         note_type: Off,
                    //         timing: 22,
                    //         voice_id: Some(0),
                    //         channel: 1,
                    //         note: 60,
                    //         velocity: 0.0,
                    //         note_length_samples: 0,
                    //     }),
                    //     VoiceTerminated(VoiceTerminatedEvent {
                    //         timing: 22,
                    //         channel: 1,
                    //         voice_id: Some(0),
                    //         note: 60,
                    //     }),
                    PreciseEventType::Note(SimpleNoteEvent {
                        note_type: NoteType::On,
                        timing: 23, // sample pos 104726
                        voice_id: Some(2),
                        channel: 1,
                        note: 60,
                        velocity: 0.8,
                        note_length_samples: 104726 as usize,
                    }),
                    PreciseEventType::Note(SimpleNoteEvent {
                        note_type: NoteType::Off,
                        timing: 22, // sample pos 104726
                        voice_id: Some(0),
                        channel: 1,
                        note: 60,
                        velocity: 0.0,
                        note_length_samples: 0 as usize, // FIXME
                    }),
                    PreciseEventType::VoiceTerminated(VoiceTerminatedEvent {
                        timing: 22, // sample pos 104726
                        voice_id: Some(0),
                        channel: 1,
                        note: 60,
                    }),
                ],
            ),
            (
                613, // Halfway through the next loop of the pattern
                vec![
                    PreciseEventType::Note(SimpleNoteEvent {
                        note_type: NoteType::On,
                        timing: 163, // sample pos 157091
                        voice_id: Some(2),
                        channel: 1,
                        note: 96,
                        velocity: 0.8,
                        note_length_samples: 104726 as usize,
                    }),
                    PreciseEventType::Note(SimpleNoteEvent {
                        note_type: NoteType::Off,
                        timing: 162, // sample pos 157090
                        voice_id: Some(1),
                        channel: 1,
                        note: 96,
                        velocity: 0.0,
                        note_length_samples: 0 as usize, // FIXME
                    }),
                    PreciseEventType::VoiceTerminated(VoiceTerminatedEvent {
                        timing: 162, // sample pos 157090
                        voice_id: Some(1),
                        channel: 1,
                        note: 96,
                    }),
                ],
            ),
        ]);
        // let left = [
        //     Note(SimpleNoteEvent {
        //         note_type: On,
        //         timing: 163,
        //         voice_id: None,
        //         channel: 1,
        //         note: 96,
        //         velocity: 0.8,
        //         note_length_samples: 104726,
        //     }),
        //     Note(SimpleNoteEvent {
        //         note_type: Off,
        //         timing: 162,
        //         voice_id: None,
        //         channel: 1,
        //         note: 96,
        //         velocity: 0.0,
        //         note_length_samples: 0,
        //     }),
        //     VoiceTerminated(VoiceTerminatedEvent {
        //         timing: 162,
        //         channel: 1,
        //         voice_id: Some(1),
        //         note: 96,
        //     }),
        // ];
        // let right = [
        //     Note(SimpleNoteEvent {
        //         note_type: On,
        //         timing: 163,
        //         voice_id: Some(2),
        //         channel: 1,
        //         note: 96,
        //         velocity: 0.8,
        //         note_length_samples: 104726,
        //     }),
        //     Note(SimpleNoteEvent {
        //         note_type: Off,
        //         timing: 162,
        //         voice_id: Some(1),
        //         channel: 1,
        //         note: 96,
        //         velocity: 0.0,
        //         note_length_samples: 0,
        //     }),
        //     VoiceTerminated(VoiceTerminatedEvent {
        //         timing: 162,
        //         channel: 1,
        //         voice_id: Some(1),
        //         note: 96,
        //     }),
        // ];
        verify_pattern_playback(&pattern, &expectations)
    }
}
