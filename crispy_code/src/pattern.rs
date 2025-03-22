use crate::dur::Dur;
use crate::lex::parse_note;
use nih_plug::nih_log;
use num::integer::lcm;
use reqwest;
use reqwest::header::CONTENT_TYPE;
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

    pub fn len(self, new_length_bars: Dur) -> NamedPattern {
        let factor = new_length_bars / self.length_bars;
        NamedPattern {
            channel: self.channel,
            events: self
                .events
                .into_iter()
                .map(|ev| Event {
                    action: ev.action,
                    dur: ev.dur * factor,
                })
                .collect(),
            length_bars: new_length_bars,
            name: String::from(self.name),
        }
    }

    pub fn note(self, expr: &str) -> NamedPattern {
        match parse_note(expr) {
            None => {
                panic!("could not parse note {}", expr);
            }
            Some((note, _, _, _)) => NamedPattern {
                channel: self.channel,
                events: self
                    .events
                    .into_iter()
                    .map(|ev| Event {
                        action: match ev.action {
                            EventType::NoteEvent(existing_note) => EventType::NoteEvent(Note {
                                note_num: note.note_num,
                                dur: existing_note.dur,
                                velocity: existing_note.velocity,
                            }),
                            _ => ev.action,
                        },
                        dur: ev.dur,
                    })
                    .collect(),
                length_bars: self.length_bars,
                name: self.name,
            },
        }
    }

    pub fn trans(self, offset: i64) -> NamedPattern {
        NamedPattern {
            channel: self.channel,
            events: self
                .events
                .into_iter()
                .map(|ev| Event {
                    action: match ev.action {
                        EventType::NoteEvent(existing_note) => EventType::NoteEvent(Note {
                            note_num: existing_note.note_num + (offset as u8),
                            dur: existing_note.dur,
                            velocity: existing_note.velocity,
                        }),
                        _ => ev.action,
                    },
                    dur: ev.dur,
                })
                .collect(),
            length_bars: self.length_bars,
            name: self.name,
        }
    }
}

pub fn start(pattern: NamedPattern) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    client
        .post(format!("http://127.0.0.1:3000/start/{}", pattern.name))
        .header(CONTENT_TYPE, "application/json")
        .json(&pattern)
        .send()?;
    Ok(())
}

pub fn stop(pattern: NamedPattern) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    client
        .post(format!("http://127.0.0.1:3000/stop/{}", pattern.name))
        .header(CONTENT_TYPE, "application/json")
        .json(&pattern)
        .send()?;
    Ok(())
}

pub fn stopall() -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    client
        .post("http://127.0.0.1:3000/stopall")
        .header(CONTENT_TYPE, "application/json")
        .send()?;
    Ok(())
}

pub fn clear(pattern: NamedPattern) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    client
        .post(format!("http://127.0.0.1:3000/clear/{}", pattern.name))
        .header(CONTENT_TYPE, "application/json")
        .json(&pattern)
        .send()?;
    Ok(())
}

pub fn clearall() -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    client
        .post("http://127.0.0.1:3000/clearall")
        .header(CONTENT_TYPE, "application/json")
        .send()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::dsl::notes;
    use crate::dur::{Dur, BAR, HALF};
    use crate::pattern::{Event, EventType, NamedPattern, Note};

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

    #[test]
    fn test_named_pattern_reverse() {
        assert_eq!(
            notes("Cx Dg").unwrap().named("foo").reverse(),
            notes("Dg Cx").unwrap().named("foo")
        );
    }

    #[test]
    fn test_named_pattern_len() {
        assert_eq!(
            notes("Cx Dg").unwrap().named("foo").len(BAR * 2),
            NamedPattern {
                channel: 1,
                events: vec![
                    Event {
                        action: EventType::NoteEvent(Note {
                            note_num: 60,
                            velocity: 0.89,
                            dur: HALF
                        }),
                        dur: BAR,
                    },
                    Event {
                        action: EventType::NoteEvent(Note {
                            note_num: 62,
                            velocity: 0.26,
                            dur: HALF
                        }),
                        dur: BAR,
                    },
                ],
                length_bars: BAR * 2,
                name: String::from("foo"),
            }
        );
    }

    #[test]
    fn test_named_pattern_note() {
        assert_eq!(
            notes("Cx Dg").unwrap().named("foo").note("Ep"),
            notes("Ex Eg").unwrap().named("foo"),
        );
    }

    #[test]
    #[should_panic]
    fn test_named_pattern_note_failure_case() {
        notes("Cx Dg").unwrap().note("(((");
    }

    #[test]
    fn test_named_pattern_trans() {
        assert_eq!(
            notes("Cx Dg").unwrap().named("foo").trans(7),
            notes("Gx Ag").unwrap().named("foo"),
        );
    }
}
