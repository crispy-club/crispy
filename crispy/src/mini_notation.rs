use crate::dur::Dur;
use crate::pattern::{Event, EventType, Note, Pattern};
use logos::Logos;
use regex::Regex;
use std::sync::LazyLock;

static DEFAULT_OCTAVE: i32 = 3;
static DEFAULT_VELOCITY: f32 = 0.8;

#[derive(Debug, Clone, PartialEq)]
struct ParseError;

fn get_pitch_class(c: char) -> i32 {
    match c {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => panic!("unknown pitch class {}", c),
    }
}

fn get_velocity(c: char) -> f32 {
    // 'a' -> 97
    // 'z' -> 122
    let ascii_code = c as u8;
    return ((((ascii_code as f32) - 96.0) / 27.0) * 100.0).round() / 100.0;
}

static NOTE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([CDEFGAB])(')?(-2|-1|0|1|2|3|4|5|6|7)?([a-z])?$").unwrap());

fn parse_note_expr(def: &str) -> Option<Note> {
    let caps = NOTE_REGEX.captures(def).unwrap();
    assert_eq!(caps.len(), 5);
    let mut note_num = get_pitch_class(caps[1].chars().next().unwrap());
    if let Some(_sharp) = caps.get(2) {
        note_num += 1
    }
    let mut octave = DEFAULT_OCTAVE;
    if let Some(matched) = caps.get(3) {
        let oct_str = matched.as_str();
        octave = oct_str.parse().expect("Could not parse octave");
    }
    note_num += 12 * (octave + 2);

    let mut velocity = DEFAULT_VELOCITY;
    if let Some(matched) = caps.get(4) {
        let vel_str = matched.as_str();
        velocity = get_velocity(vel_str.chars().next().unwrap());
    }
    Some(Note {
        note_num: note_num as u8,
        velocity: velocity,
        // Duration is specified as ratio relative to the containing event's duration.
        // Event duration is really what determines the rhythm of the overall pattern.
        dur: Dur::new(1, 2),
    })
}

#[derive(Clone, Debug, Logos, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
enum Token {
    #[token("[")]
    GroupStart,
    #[token("]")]
    GroupEnd,
    #[regex(r"[CDEFGAB](')?(-2|-1|0|1|2|3|4|5|6|7)?([a-z])?", |lex| parse_note_expr(lex.slice()))]
    NoteExpr(Note),
}

#[derive(Clone, Debug, PartialEq)]
enum Pelem {
    Note(Note),
    Group(Vec<Pelem>),
}

fn transform<'source>(root: Pelem, len_bars: Dur) -> Vec<Event> {
    let mut events: Vec<Event> = vec![];
    transform_r(root, len_bars, &mut events);
    events
}

fn transform_r<'source>(root: Pelem, len: Dur, events: &mut Vec<Event>) {
    match root {
        Pelem::Note(note) => events.push(Event {
            action: EventType::NoteEvent(note),
            dur: len,
        }),
        Pelem::Group(elems) => {
            if elems.len() == 0 {
                return;
            }
            let num_elems = elems.len();
            let each_dur = len.div_int(num_elems as i64);
            for elem in elems {
                transform_r(elem, each_dur, events);
            }
        }
    }
}

fn pat(def: &str) -> Result<Pattern, ParseError> {
    let len_bars = Dur::new(1, 1);
    Ok(Pattern {
        channel: None,
        events: get_events(def, len_bars),
        length_bars: Some(len_bars),
    })
}

fn get_events(def: &str, len_bars: Dur) -> Vec<Event> {
    let root_elem = get_root_elem(def);
    return transform(root_elem, len_bars);
}

fn get_root_elem(def: &str) -> Pelem {
    let tokens: Vec<Token> = Token::lexer(def).map(|res| res.unwrap()).collect();
    let mut root = Pelem::Group(vec![]);
    get_elems_r(&mut root, tokens);
    root
}

fn get_elems_r(root: &mut Pelem, tokens: Vec<Token>) -> usize {
    // Handle the empty string.
    if tokens.len() == 0 {
        return 0;
    }
    // `[C]` is the same thing as `C`
    if tokens.len() >= 2
        && tokens[0] == Token::GroupStart
        && tokens[tokens.len() - 1] == Token::GroupEnd
    {
        return get_elems_r(root, (&tokens[1..tokens.len() - 1]).to_vec());
    }
    match root {
        Pelem::Note(_) => 0,
        Pelem::Group(ref mut elems) => {
            let mut idx = 0;
            while idx < tokens.len() {
                let tok = &tokens[idx];

                match tok {
                    Token::GroupStart => {
                        let mut sub_group = Pelem::Group(vec![]);
                        let jump = get_elems_r(&mut sub_group, (&tokens[(idx + 1)..]).to_vec());
                        elems.push(sub_group);
                        idx += jump;
                        continue;
                    }
                    Token::GroupEnd => {
                        idx += 1;
                        break;
                    }
                    Token::NoteExpr(note) => {
                        elems.push(Pelem::Note(*note));
                    }
                }
                idx += 1;
            }
            idx
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dur::{Dur, BAR};
    use crate::mini_notation::{
        get_root_elem, get_velocity, parse_note_expr, pat, Pelem, DEFAULT_VELOCITY, NOTE_REGEX,
    };
    use crate::pattern::{Event, EventType, Note, Pattern};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pattern_empty() {
        assert_eq!(
            pat("[]"),
            Ok(Pattern {
                channel: None,
                length_bars: Some(BAR),
                events: vec![],
            }),
        );
    }

    #[test]
    fn test_get_velocity() {
        let cases: Vec<(char, f32)> = vec![
            ('a', 0.04),
            ('b', 0.07),
            ('c', 0.11),
            ('d', 0.15),
            ('e', 0.19),
            ('f', 0.22),
            ('g', 0.26),
            ('h', 0.3),
            ('i', 0.33),
            ('j', 0.37),
            ('k', 0.41),
            ('l', 0.44),
            ('m', 0.48),
            ('n', 0.52),
            ('o', 0.56),
            ('p', 0.59),
            ('q', 0.63),
            ('r', 0.67),
            ('s', 0.7),
            ('t', 0.74),
            ('u', 0.78),
            ('v', 0.81),
            ('w', 0.85),
            ('x', 0.89),
            ('y', 0.93),
            ('z', 0.96),
        ];
        for case in cases {
            assert_eq!(case.1, get_velocity(case.0));
        }
    }

    #[test]
    fn test_pattern_single_note() {
        assert_eq!(
            pat("[Cx]"),
            Ok(Pattern {
                channel: None,
                length_bars: Some(BAR),
                events: vec![Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 60,
                        velocity: 0.89,
                        dur: Dur::new(1, 2),
                    }),
                    dur: Dur::new(1, 1),
                },],
            }),
        );
    }

    #[test]
    fn test_pattern_two_notes() {
        assert_eq!(
            pat("[Cx D'g]"),
            Ok(Pattern {
                channel: None,
                length_bars: Some(BAR),
                events: vec![
                    Event {
                        action: EventType::NoteEvent(Note {
                            note_num: 60,
                            velocity: 0.89,
                            dur: Dur::new(1, 2),
                        }),
                        dur: Dur::new(1, 2),
                    },
                    Event {
                        action: EventType::NoteEvent(Note {
                            note_num: 63,
                            velocity: 0.26,
                            dur: Dur::new(1, 2),
                        }),
                        dur: Dur::new(1, 2),
                    },
                ],
            }),
        );
    }

    #[test]
    fn test_pattern_with_subpattern() {
        let actual = pat("[Cx [D'g G4u]]");
        let expect = Pattern {
            channel: None,
            length_bars: Some(BAR),
            events: vec![
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 60,
                        velocity: 0.89,
                        dur: Dur::new(1, 2),
                    }),
                    dur: Dur::new(1, 2),
                },
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 63,
                        velocity: 0.26,
                        dur: Dur::new(1, 2),
                    }),
                    dur: Dur::new(1, 4),
                },
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 79,
                        velocity: 0.78,
                        dur: Dur::new(1, 2),
                    }),
                    dur: Dur::new(1, 4),
                },
            ],
        };
        assert_eq!(actual, Ok(expect));
    }

    #[test]
    fn test_note_regex() {
        let caps = NOTE_REGEX.captures("C'").unwrap();
        assert_eq!(caps.len(), 5);
        assert_eq!(&caps[1], "C");
        assert_eq!(&caps[2], "'");

        let caps = NOTE_REGEX.captures("C3").unwrap();
        assert_eq!(caps.len(), 5);
        assert_eq!(&caps[1], "C");
        assert_eq!(&caps[3], "3");

        let caps = NOTE_REGEX.captures("Cx").unwrap();
        assert_eq!(caps.len(), 5);
        assert_eq!(&caps[1], "C");
        assert_eq!(&caps[4], "x");

        let caps = NOTE_REGEX.captures("C'3").unwrap();
        assert_eq!(caps.len(), 5);
        assert_eq!(&caps[1], "C");
        assert_eq!(&caps[2], "'");
        assert_eq!(&caps[3], "3");

        let caps = NOTE_REGEX.captures("C'x").unwrap();
        assert_eq!(caps.len(), 5);
        assert_eq!(&caps[1], "C");
        assert_eq!(&caps[2], "'");
        assert_eq!(&caps[4], "x");

        let caps = NOTE_REGEX.captures("C'3x").unwrap();
        assert_eq!(caps.len(), 5);
        assert_eq!(&caps[1], "C");
        assert_eq!(&caps[2], "'");
        assert_eq!(&caps[3], "3");
        assert_eq!(&caps[4], "x");
    }

    #[test]
    fn test_parse_note_expr() {
        let note = parse_note_expr("C");
        assert_eq!(
            note,
            Some(Note {
                note_num: 60,
                velocity: DEFAULT_VELOCITY,
                dur: Dur::new(1, 2),
            })
        );

        let note = parse_note_expr("C'");
        assert_eq!(
            note,
            Some(Note {
                note_num: 61,
                velocity: DEFAULT_VELOCITY,
                dur: Dur::new(1, 2),
            })
        );

        let note = parse_note_expr("E1");
        assert_eq!(
            note,
            Some(Note {
                note_num: 40,
                velocity: DEFAULT_VELOCITY,
                dur: Dur::new(1, 2),
            })
        );

        let note = parse_note_expr("D'2");
        assert_eq!(
            note,
            Some(Note {
                note_num: 51,
                velocity: DEFAULT_VELOCITY,
                dur: Dur::new(1, 2),
            })
        );
    }

    #[test]
    fn test_get_root_elem() {
        let elem = get_root_elem("[C]");
        assert_eq!(
            elem,
            Pelem::Group(vec![Pelem::Note(Note {
                note_num: 60,
                velocity: DEFAULT_VELOCITY,
                dur: Dur::new(1, 2),
            })])
        );
    }

    #[test]
    fn test_get_root_elem_subgroup() {
        let elem = get_root_elem("C [D E]");
        assert_eq!(
            elem,
            Pelem::Group(vec![
                Pelem::Note(Note {
                    note_num: 60,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
                Pelem::Group(vec![
                    Pelem::Note(Note {
                        note_num: 62,
                        velocity: DEFAULT_VELOCITY,
                        dur: Dur::new(1, 2),
                    }),
                    Pelem::Note(Note {
                        note_num: 64,
                        velocity: DEFAULT_VELOCITY,
                        dur: Dur::new(1, 2),
                    }),
                ]),
            ])
        );
    }
}
