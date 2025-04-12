#![allow(dead_code)]
#![allow(non_upper_case_globals)]

use crate::dsl::notes;
use crate::parse::ParseError;
use crate::pattern::{Event, EventType, NamedPattern, Note};
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
fn get_key(def: &str) -> u8 {
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

// This is a bit verbose and could use some improvement
// To generate a pattern this way, the rhai code would look
// something like this
//
// scale("x t").scale("C", acoustic)
//
// What would be a cleaner API here?
//
// scale("C", "x t", acoustic)
// scali("C", "x t", acoustic, [2, 3, 1, 4])
//
// In this approach, the first token in the pattern is actually the key.
// We would then parse the rest of the pattern as a normal NamedPattern.
//
// One thing that's difficult to figure out is how to have the rust
// functions not be completely tied to rhai. I guess I can do the conversion
// when I register the function(s)
// For example, I kind of like registering the scale data as Vec<Dynamic>
// This gives you the ability to iterate over the values in rhai and
// makes the data structure feel intuitive
// But then when a user passes a scale into a rust function it will need
// to be converted from Vec<Dynamic> back to &[u8]
//
pub fn scale(key: &str, def: &str, scl: Vec<u8>) -> Result<NamedPattern, ParseError> {
    let pattern = notes(def)?;
    Ok(ScalePattern::WithDefaultIndex(pattern).update_notes(get_key(key), &scl))
}

pub fn scali(
    key: &str,
    def: &str,
    scl: Vec<u8>,
    idx: Vec<usize>,
) -> Result<NamedPattern, ParseError> {
    let pattern = notes(def)?;
    Ok(ScalePattern::WithCustomIndex(pattern, idx).update_notes(get_key(key), &scl))
}

/// Forces the notes of a pattern to conform to a scale.
enum ScalePattern {
    WithDefaultIndex(NamedPattern),
    WithCustomIndex(NamedPattern, Vec<usize>),
}

impl ScalePattern {
    /// Panics if key >= 12
    pub fn update_notes(&self, key: u8, scl: &Vec<u8>) -> NamedPattern {
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
    scl: &Vec<u8>,
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
    scl: &Vec<u8>,
    idx: &Vec<usize>,
) -> Vec<Event> {
    assert!(key < 12);
    let mut sev = Vec::<Event>::with_capacity(events.len());
    let mut scl_idx = 0 as usize;
    for event in events.into_iter() {
        sev.push(compute_scale_event(event, key + (scl[idx[scl_idx]] % 12)));
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

fn default_indices(scl: &Vec<u8>) -> Vec<usize> {
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
            let key = "D";
            let pitch_classes = scl.clone();
            let pat = pitch_classes
                .clone()
                .into_iter()
                .map(|_| "x")
                .collect::<Vec<&str>>()
                .as_slice()
                .join(" ");
            let actual_pattern = scale(key, pat.as_str(), scl.clone()).unwrap().named("foo");
            let note_nums = pitch_classes
                .clone()
                .into_iter()
                .map(|pc| (pc as u8) + get_key(key) + (((DEFAULT_OCTAVE as u8) + 2) * 12))
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

    #[test]
    fn test_get_key() {
        assert_eq!(get_key("C"), 0);
        assert_eq!(get_key("C'"), 1);
        assert_eq!(get_key("D"), 2);
        assert_eq!(get_key("D'"), 3);
        assert_eq!(get_key("E"), 4);
        assert_eq!(get_key("F"), 5);
        assert_eq!(get_key("F'"), 6);
        assert_eq!(get_key("G"), 7);
        assert_eq!(get_key("G'"), 8);
        assert_eq!(get_key("A"), 9);
        assert_eq!(get_key("A'"), 10);
        assert_eq!(get_key("B"), 11);
    }

    #[test]
    #[should_panic]
    fn test_get_key_panic() {
        get_key("X");
    }

    #[test]
    fn test_scale_with_custom_index() {
        let key = "G";
        let pattern = scali(
            key,
            "x e t [f p]",
            Scales.get("hirajoshi").unwrap().clone(),
            // ("hirajoshi", vec![0, 4, 6, 7, 11]),
            vec![1, 3, 2, 0, 3],
        )
        .unwrap()
        .named("foo");
        assert_eq!(
            pattern,
            NamedPattern {
                channel: 1,
                events: vec![
                    (71, 0.89, 4),
                    (74, 0.19, 4),
                    (73, 0.74, 4),
                    (67, 0.22, 8),
                    (74, 0.59, 8)
                ]
                .into_iter()
                .map(|t| Event {
                    action: EventType::NoteEvent(Note {
                        note_num: t.0,
                        velocity: t.1,
                        dur: Dur::new(1, 2),
                    }),
                    dur: Dur::new(1, t.2),
                })
                .collect(),
                length_bars: Dur::new(1, 1),
                name: String::from("foo"),
            }
        );
    }

    #[test]
    fn test_scale_with_custom_index_and_not_default_octave_values() {
        let key = "G";
        let pattern = scali(
            key,
            "0x 1e 5t [2f 6p]",
            Scales.get("hirajoshi").unwrap().clone(),
            // ("hirajoshi", vec![0, 4, 6, 7, 11]),
            vec![1, 3, 2, 0, 3],
        )
        .unwrap()
        .named("foo");
        assert_eq!(
            pattern,
            NamedPattern {
                channel: 1,
                events: vec![
                    (71 - (12 * 3), 0.89, 4),
                    (74 - (12 * 2), 0.19, 4),
                    (73 + (12 * 2), 0.74, 4),
                    (67 - 12, 0.22, 8),
                    (74 + (12 * 3), 0.59, 8)
                ]
                .into_iter()
                .map(|t| Event {
                    action: EventType::NoteEvent(Note {
                        note_num: t.0,
                        velocity: t.1,
                        dur: Dur::new(1, 2),
                    }),
                    dur: Dur::new(1, t.2),
                })
                .collect(),
                length_bars: Dur::new(1, 1),
                name: String::from("foo"),
            }
        );
    }
}
