use num::integer::lcm;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Note {
    pub note_num: u8,
    pub dur_ms: i64,
    pub velocity: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum EventType {
    Rest,
    NoteEvent(Note),
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct FractionalDuration {
    pub num: i64,
    pub den: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Event {
    pub action: EventType,
    pub dur_frac: FractionalDuration,
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
            .map(|event| event.dur_frac.den)
            .reduce(|acc, e| lcm(acc, e))
            .unwrap();
        self.events = self
            .events
            .clone()
            .into_iter()
            .map(|event| {
                let mut clone = event.clone();
                let multiplier = least_common_multiple / event.dur_frac.den;
                clone.dur_frac = FractionalDuration {
                    num: event.dur_frac.num * multiplier,
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

        let samples_per_milli = sample_rate / 1000.0;
        let mut sample_idx: usize = 0;
        let mut events_map: HashMap<usize, Vec<SimpleNoteEvent>> = HashMap::new();

        for (idx, event) in pattern.events.clone().into_iter().enumerate() {
            match event.action {
                EventType::NoteEvent(note) => {
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
                    let note_off_timing = (sample_idx
                        + ((samples_per_milli as f64) * (note.dur_ms as f64)) as usize)
                        as u32;

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
                EventType::Rest => {
                    events_map.insert(
                        sample_idx,
                        vec![SimpleNoteEvent {
                            note_type: NoteType::Rest,
                            timing: 0,
                            voice_id: None,
                            channel: 1,
                            note: 0,
                            velocity: 0.0,
                        }],
                    );
                }
            }
            sample_idx +=
                ((tick_length_samples * event.dur_frac.num) + extra_samples[idx]) as usize;
        }
        return PrecisePattern {
            events: events_map,
            length_samples: sample_idx,
            playing: playing,
        };
    }

    pub fn get_events(&self, start: usize, end: usize) -> Vec<SimpleNoteEvent> {
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
