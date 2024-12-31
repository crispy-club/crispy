use num::integer::lcm;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct FractionalDuration {
    pub num: i64,
    pub den: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Note {
    pub note_num: u8,
    pub dur: FractionalDuration,
    pub velocity: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum EventType {
    Rest,
    NoteEvent(Note),
    MultiNoteEvent(Vec<Note>),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Event {
    pub action: EventType,
    pub dur: FractionalDuration,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pattern {
    pub events: Vec<Event>,
    pub length_bars: Option<FractionalDuration>,
}

impl Pattern {
    fn compute_events_lcm(&mut self) -> i64 {
        let least_common_multiple = self
            .events
            .clone()
            .into_iter()
            .map(|event| event.dur.den)
            .reduce(|acc, e| lcm(acc, e))
            .unwrap();
        self.events = self
            .events
            .clone()
            .into_iter()
            .map(|event| {
                let mut clone = event.clone();
                let multiplier = least_common_multiple / event.dur.den;
                clone.dur = FractionalDuration {
                    num: event.dur.num * multiplier,
                    den: least_common_multiple,
                };
                clone
            })
            .collect();
        least_common_multiple
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NamedPattern {
    pub events: Vec<Event>,
    pub length_bars: Option<FractionalDuration>,
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NoteType {
    On,
    Off,
    Rest,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimpleNoteEvent {
    pub note_type: NoteType,
    pub timing: u32,
    pub voice_id: Option<i32>,
    pub channel: u8,
    pub note: u8,
    pub velocity: f32,
}

#[derive(Clone)]
pub struct PrecisePattern {
    pub events: HashMap<usize, Vec<SimpleNoteEvent>>,
    pub length_samples: usize,
    pub playing: bool,
    pub notes_playing: HashMap<(u8, u8, u8), ()>,
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
    events_map: &mut HashMap<usize, Vec<SimpleNoteEvent>>,
    event: &Event,
    note: &Note,
    tick_length_samples: i64,
    sample_idx: usize,
) {
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
    let event_length_samples = event.dur.num * tick_length_samples;
    let note_length_samples = (note.dur.num * event_length_samples) / note.dur.den;
    let note_off_timing = ((sample_idx as i64) + note_length_samples) as u32;

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

fn insert_event(
    events_map: &mut HashMap<usize, Vec<SimpleNoteEvent>>,
    event: &Event,
    tick_length_samples: i64,
    sample_idx: usize,
) {
    match &event.action {
        EventType::MultiNoteEvent(notes) => {
            for note in notes {
                insert_note(events_map, event, &note, tick_length_samples, sample_idx);
            }
        }
        EventType::NoteEvent(note) => {
            insert_note(events_map, event, &note, tick_length_samples, sample_idx);
        }
        EventType::Rest => {
            events_map.insert(
                sample_idx,
                vec![SimpleNoteEvent {
                    note_type: NoteType::Rest,
                    timing: sample_idx as u32,
                    voice_id: None,
                    channel: 1,
                    note: 0,
                    velocity: 0.0,
                }],
            );
        }
    }
}

impl PrecisePattern {
    pub fn from(
        pattern: &mut Pattern,
        sample_rate: f32,
        tempo: f64,
        playing: bool,
    ) -> PrecisePattern {
        let least_common_multiple = pattern.compute_events_lcm();
        let samples_per_bar = (sample_rate as f64 * (240.0 / tempo)) as i64;
        let pattern_length_samples =
            (pattern.length_bars.unwrap().num * samples_per_bar) / pattern.length_bars.unwrap().den;
        let tick_length_samples = pattern_length_samples / least_common_multiple;
        let samples_remainder = pattern_length_samples % least_common_multiple;
        let extra_samples = compute_extra_samples(samples_remainder, pattern.events.len());

        let mut sample_idx: usize = 0;
        let mut events_map: HashMap<usize, Vec<SimpleNoteEvent>> = HashMap::new();

        for (idx, event) in pattern.events.clone().into_iter().enumerate() {
            insert_event(&mut events_map, &event, tick_length_samples, sample_idx);
            sample_idx += ((tick_length_samples * event.dur.num) + extra_samples[idx]) as usize;
        }
        return PrecisePattern {
            events: events_map,
            length_samples: sample_idx,
            playing: playing,
            notes_playing: HashMap::new(),
        };
    }

    pub fn get_events(&mut self, start: usize, end: usize) -> Vec<SimpleNoteEvent> {
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

    pub fn get_events_adj(
        &mut self,
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
                    events.into_iter().for_each(|ev| match ev.note_type {
                        NoteType::Rest => {}
                        NoteType::Off => {
                            self.notes_playing.remove(&(
                                ev.channel,
                                ev.note,
                                ev.voice_id.unwrap_or(0) as u8,
                            ));
                        }
                        NoteType::On => {
                            self.notes_playing
                                .insert((ev.channel, ev.note, ev.voice_id.unwrap_or(0) as u8), ());
                        }
                    });
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

    pub fn get_notes_playing(&mut self) -> Vec<SimpleNoteEvent> {
        let notes_playing = self
            .notes_playing
            .keys()
            .map(|k| SimpleNoteEvent {
                note_type: NoteType::Off,
                timing: 0,
                voice_id: Some(k.2 as i32),
                channel: k.0,
                note: k.1,
                velocity: 0.0,
            })
            .collect();
        self.notes_playing = HashMap::new();
        notes_playing
    }
}

#[cfg(test)]
mod tests {
    use crate::pattern::{
        compute_extra_samples, Event, EventType, FractionalDuration, Note, NoteType, Pattern,
        PrecisePattern, SimpleNoteEvent,
    };
    use std::collections::HashMap;

    #[test]
    fn test_fractional_duration_clone() {
        let dur = FractionalDuration { num: 1, den: 4 };
        let clone = dur.clone();
        assert_eq!(dur, clone);
    }

    #[test]
    fn test_note_clone() {
        let note = Note {
            note_num: 60,
            velocity: 0.5,
            dur: FractionalDuration { num: 1, den: 4 },
        };
        let clone = note.clone();
        assert_eq!(note, clone);
    }

    #[test]
    fn test_event_clone() {
        let event = Event {
            action: EventType::NoteEvent(Note {
                note_num: 60,
                velocity: 0.5,
                dur: FractionalDuration { num: 1, den: 4 },
            }),
            dur: FractionalDuration { num: 1, den: 2 },
        };
        let clone = event.clone();
        assert_eq!(event, clone);
    }

    #[test]
    fn test_compute_extra_samples() -> Result<(), String> {
        let extra_samples = compute_extra_samples(37, 5);
        assert_eq!(extra_samples, vec![8, 8, 7, 7, 7]);
        Ok(())
    }

    #[test]
    fn test_precise_pattern() -> Result<(), String> {
        let pattern = Pattern {
            length_bars: Some(FractionalDuration { num: 1, den: 2 }),
            events: vec![
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 60,
                        velocity: 0.8,
                        dur: FractionalDuration { num: 1, den: 4 },
                    }),
                    dur: FractionalDuration { num: 1, den: 2 },
                },
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 96,
                        velocity: 0.8,
                        dur: FractionalDuration { num: 1, den: 4 },
                    }),
                    dur: FractionalDuration { num: 1, den: 2 },
                },
            ],
        };
        let sample_rate = 48000 as f32;
        let tempo = 110 as f64;
        let mut precise_pattern =
            PrecisePattern::from(&mut pattern.clone(), sample_rate, tempo, true);
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
                25,
                vec![SimpleNoteEvent {
                    note_type: NoteType::Off,
                    timing: 145,
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
                    timing: 70,
                    voice_id: None,
                    channel: 1,
                    note: 96,
                    velocity: 0.8,
                }],
            ),
            (
                127,
                vec![SimpleNoteEvent {
                    note_type: NoteType::Off,
                    timing: 215,
                    voice_id: None,
                    channel: 1,
                    note: 96,
                    velocity: 0.0,
                }],
            ),
            // Add this last one to see how the code behaves when the pattern loops.
            (
                204,
                vec![SimpleNoteEvent {
                    note_type: NoteType::On,
                    timing: 139,
                    voice_id: None,
                    channel: 1,
                    note: 60,
                    velocity: 0.8,
                }],
            ),
        ]);
        let max_buf_num = *expectations.keys().max().unwrap();

        for bufnum in 0..(max_buf_num + 1) {
            let expected_events_opt = expectations.get(&bufnum);
            let buffer_start_samples = bufnum * buffer_size_samples;
            let buffer_end_samples = buffer_start_samples + buffer_size_samples;
            let events = precise_pattern.get_events(buffer_start_samples, buffer_end_samples);
            match expected_events_opt {
                Some(expected_events) => {
                    assert_eq!(*expected_events, events);
                }
                None => {
                    assert_eq!(events.len(), 0);
                }
            }
        }
        Ok(())
    }
}
