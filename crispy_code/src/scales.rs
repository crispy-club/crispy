#![allow(dead_code)]
#![allow(non_upper_case_globals)]

use crate::dsl::notes;
use crate::parse::ParseError;
use crate::pattern::{Event, EventType, NamedPattern, Note};
use rhai::Engine;
use std::collections::HashMap;

/// Panics if key in not in this list
/// * `"C"`
/// * `"C'"`
/// * `"D"`
/// * `"D'"`
/// * `"E"`
/// * `"F"`
/// * `"F'"`
/// * `"G"`
/// * `"G'"`
/// * `"A"`
/// * `"A'"`
/// * `"B"`
fn key(def: &str) -> u8 {
    match def {
        "C" => 0,
        "C'" => 1,
        "D" => 2,
        "D'" => 3,
        "E" => 4,
        "F" => 5,
        "F'" => 6,
        "G" => 7,
        "G'" => 8,
        "A" => 9,
        "A'" => 10,
        "B" => 11,
        _ => panic!("unknown key: {:?}", def),
    }
}

pub fn scale(def: &str) -> Result<ScalePattern, ParseError> {
    let pattern = notes(def)?;
    Ok(ScalePattern::WithDefaultIndex(pattern))
}

/// Forces the notes of a pattern to conform to a scale.
pub enum ScalePattern {
    WithDefaultIndex(NamedPattern),
    WithCustomIndex(NamedPattern, Vec<usize>),
}

impl ScalePattern {
    /// Panics if key >= 12
    pub fn scale(&self, key: u8, scl: &'static [u8]) -> NamedPattern {
        match self {
            ScalePattern::WithDefaultIndex(pat) => {
                let idxs = default_indices(&scl);
                compute_scale_pattern(pat, key, scl, &idxs)
            }
            ScalePattern::WithCustomIndex(pat, idx) => compute_scale_pattern(pat, key, &scl, idx),
        }
    }

    pub fn index(&self, idx: Vec<usize>) -> Self {
        match self {
            ScalePattern::WithDefaultIndex(pat) => ScalePattern::WithCustomIndex(pat.clone(), idx),
            ScalePattern::WithCustomIndex(pat, _) => {
                ScalePattern::WithCustomIndex(pat.clone(), idx)
            }
        }
    }
}

fn compute_scale_pattern(
    pat: &NamedPattern,
    key: u8,
    scl: &'static [u8],
    idx: &Vec<usize>,
) -> NamedPattern {
    assert!(key < 12);
    NamedPattern {
        channel: pat.channel,
        events: compute_scale_events(&pat.events, key, scl, idx),
        length_bars: pat.length_bars,
        name: pat.name.clone(),
    }
}

fn compute_scale_events(
    events: &Vec<Event>,
    key: u8,
    scl: &'static [u8],
    idx: &Vec<usize>,
) -> Vec<Event> {
    assert!(key < 12);
    let mut sev = Vec::<Event>::with_capacity(events.len());
    let mut scl_idx = 0 as usize;
    for event in events.into_iter() {
        sev.push(compute_scale_event(
            event,
            // key + (scl.pitchclasses[idx[scl_idx]] % 12),
            key + (scl[idx[scl_idx]] % 12),
        ));
        scl_idx = (scl_idx + 1) % idx.len();
    }
    sev
}

fn compute_scale_event(event: &Event, pitchclass: u8) -> Event {
    Event {
        action: match &event.action {
            EventType::NoteEvent(note) => {
                let octave = note.note_num / (12 as u8);
                EventType::NoteEvent(Note {
                    note_num: pitchclass + (octave * 12),
                    velocity: note.velocity,
                    dur: note.dur,
                })
            }
            e => e.clone(),
        },
        dur: event.dur,
    }
}

fn default_indices(scl: &'static [u8]) -> Vec<usize> {
    (0..scl.len()).map(|x| x as usize).collect()
}

lazy_static::lazy_static! {
    pub static ref Scales: HashMap<&'static str, Vec<u8>> = HashMap::from([
    ("acoustic", vec![0, 2, 4, 6, 7, 9, 10]),
    ("altered", vec![0, 1, 3, 4, 6, 8, 10]),
    ("augmented", vec![0, 3, 4, 7, 8, 11]),
    ("bebop", vec![0, 2, 4, 5, 7, 9, 10, 11]),
    ("blues", vec![0, 3, 5, 6, 7, 10]),
    ("chromatic", vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
    ("dorian", vec![0, 2, 3, 5, 7, 9, 10]),
    ("double_harm", vec![0, 1, 4, 5, 7, 8, 11]),
    ("enigmatic", vec![0, 1, 4, 6, 8, 10, 11]),
    ("flamenco", vec![0, 1, 4, 5, 7, 8, 11]),
    ("gypsy", vec![0, 2, 3, 6, 7, 8, 10]),
    ("half_diminished", vec![0, 2, 3, 5, 6, 8, 10]),
    ("hirajoshi", vec![0, 4, 6, 7, 11]),
    ("insen", vec![0, 1, 5, 7, 10]),
    ("iwato", vec![0, 1, 5, 6, 10]),
    ("locrian", vec![0, 1, 3, 5, 6, 8, 10]),
    ("locrian_sharp6", vec![0, 1, 3, 5, 6, 9, 10]),
    ("lydian", vec![0, 2, 4, 6, 7, 9, 11]),
    ("lydian_augmented", vec![0, 2, 4, 6, 8, 9, 11]),
    ("lydian_diminished", vec![0, 2, 3, 6, 7, 9, 11]),
    ("ionian", vec![0, 2, 4, 5, 7, 9, 11]),
    ("maj", vec![0, 2, 4, 5, 7, 9, 11]),
    ("maj_harm", vec![0, 2, 4, 5, 7, 8, 11]),
    ("maj_hungarian", vec![0, 3, 4, 6, 7, 9, 10]),
    ("maj_locrian", vec![0, 2, 4, 5, 6, 8, 10]),
    ("maj_neapolitan", vec![0, 1, 3, 5, 7, 9, 11]),
    ("maj_pent", vec![0, 2, 4, 7, 9]),
    ("min_harm", vec![0, 2, 3, 5, 7, 8, 11]),
    ("min_hungarian", vec![0, 2, 3, 6, 7, 8, 11]),
    ("min_melodic", vec![0, 2, 3, 5, 7, 9, 11]),
    ("min_nat", vec![0, 2, 3, 5, 7, 8, 10]),
    ("min_neapolitan", vec![0, 1, 3, 5, 7, 8, 11]),
    ("min_pent", vec![0, 3, 5, 7, 10]),
    ("mixolydian", vec![0, 2, 4, 5, 7, 9, 10]),
    ("octatonic", vec![0, 2, 3, 5, 6, 8, 9, 11]),
    ("persian", vec![0, 1, 4, 5, 6, 8, 11]),
    ("phrygian", vec![0, 1, 3, 5, 7, 8, 10]),
    ("phrygian_dominant", vec![0, 1, 4, 5, 7, 8, 10]),
    ("prometheus", vec![0, 2, 4, 6, 9, 10]),
    ("tritone", vec![0, 1, 4, 6, 7, 10]),
    ("tritone_semi2", vec![0, 1, 2, 6, 7, 8]),
    ("ukrainian_dorian", vec![0, 2, 3, 6, 7, 9, 10]),
    ("whole_tone", vec![0, 2, 4, 6, 8, 10]),
    ("yo", vec![0, 2, 5, 7, 9]),
    ]);
}

fn register_scales_for_scripting(engine: &mut Engine) {
    for (scale_name, pitch_classes) in Scales.iter() {
        engine.register_global_module(rhai::Shared::new({
            let mut module = rhai::Module::new();
            module.set_var(*scale_name, pitch_classes);
            module
        }));
    }
}

#[cfg(test)]
mod tests {
    use crate::dur::Dur;
    use crate::lex::DEFAULT_OCTAVE;
    use crate::pattern::{Event, EventType, NamedPattern, Note};
    use crate::scales::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_all_the_scales() {
        for scl in Scales.values() {
            let dee = key("D");
            let pitch_classes = scl.clone();
            let pat = pitch_classes
                .clone()
                .into_iter()
                .map(|_| "x")
                .collect::<Vec<&str>>()
                .as_slice()
                .join(" ");
            let actual_pattern = scale(pat.as_str()).unwrap().scale(dee, scl).named("foo");
            let note_nums = pitch_classes
                .clone()
                .into_iter()
                .map(|pc| (pc as u8) + dee + (((DEFAULT_OCTAVE as u8) + 2) * 12))
                .collect::<Vec<u8>>();
            assert_eq!(
                actual_pattern,
                NamedPattern {
                    channel: 1,
                    events: note_nums
                        .into_iter()
                        .map(|note_num| Event {
                            action: EventType::NoteEvent(Note {
                                note_num: note_num,
                                velocity: 0.89,
                                dur: Dur::new(1, 2),
                            }),
                            dur: Dur::new(1, pitch_classes.len() as i64),
                        })
                        .collect(),
                    length_bars: Dur::new(1, 1),
                    name: String::from("foo"),
                },
            );
        }
    }
}
