use crate::dur::Dur;
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

#[derive(Clone, CustomType, Debug, Deserialize, PartialEq, Serialize)]
pub struct NamedPattern {
    pub channel: u8,
    pub events: Vec<Event>,
    pub length_bars: Dur,
    pub name: String,
}

impl NamedPattern {
    pub fn named(&self, name: &str) -> NamedPattern {
        NamedPattern {
            channel: self.channel,
            events: self.events.clone(),
            length_bars: self.length_bars,
            name: String::from(name),
        }
    }
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
