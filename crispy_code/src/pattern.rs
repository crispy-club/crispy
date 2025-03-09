use crate::dur::Dur;
use nih_plug::nih_log;
use num::integer::lcm;
use rhai::{CustomType, TypeBuilder};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, CustomType, Debug, Deserialize, PartialEq, Serialize)]
pub struct Note {
    pub note_num: u8,
    pub dur: Dur,
    pub velocity: f32,
}

#[derive(Clone, Copy, CustomType, Debug, Deserialize, PartialEq, Serialize)]
pub struct CtrlEvent {
    pub cc: u8,
    pub value: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum EventType {
    Rest,
    NoteEvent(Note),
    MultiNoteEvent(Vec<Note>),
    Ctrl(CtrlEvent),
}

#[derive(Clone, CustomType, Debug, Deserialize, PartialEq, Serialize)]
pub struct Event {
    pub action: EventType,
    pub dur: Dur,
}

#[derive(Clone, CustomType, Debug, Deserialize, PartialEq, Serialize)]
pub struct Pattern {
    pub channel: u8,
    pub events: Vec<Event>,
    pub length_bars: Dur,
}

impl Pattern {
    pub fn compute_events_lcm(&mut self) -> i64 {
        nih_log!("computing lcm of events {:?}", self.events.clone());
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
                clone.dur = Dur {
                    num: event.dur.num * multiplier,
                    den: least_common_multiple,
                };
                clone
            })
            .collect();
        least_common_multiple
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NamedPattern {
    pub channel: u8,
    pub events: Vec<Event>,
    pub length_bars: Dur,
    pub name: String,
}

impl NamedPattern {
    pub fn named(self, name: &str) -> NamedPattern {
        NamedPattern {
            channel: self.channel,
            events: self.events.clone(),
            length_bars: self.length_bars,
            name: String::from(name),
        }
    }

    pub fn reverse(self) -> NamedPattern {
        NamedPattern {
            channel: self.channel,
            events: self.events.into_iter().rev().collect(),
            length_bars: self.length_bars,
            name: String::from(self.name),
        }
    }

    pub fn stretch(self, new_length_bars: Dur) -> NamedPattern {
        NamedPattern {
            channel: self.channel,
            events: events_stretch(self.events, self.length_bars, new_length_bars),
            length_bars: new_length_bars,
            name: String::from(self.name),
        }
    }
}

fn events_stretch(events: Vec<Event>, old_length: Dur, new_length: Dur) -> Vec<Event> {
    let factor = new_length / old_length;
    events
        .into_iter()
        .map(|ev| Event {
            action: ev.action,
            dur: ev.dur * factor,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::dur::Dur;
    use crate::pattern::{Event, EventType, Note};

    #[test]
    fn test_note_clone() {
        let note = Note {
            note_num: 60,
            velocity: 0.5,
            dur: Dur { num: 1, den: 4 },
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
                dur: Dur { num: 1, den: 4 },
            }),
            dur: Dur { num: 1, den: 2 },
        };
        let clone = event.clone();
        assert_eq!(event, clone);
    }
}
