use crate::dur::Dur;
use crate::pattern::Note;
use logos::Logos;
use regex::Regex;
use std::sync::LazyLock;

pub static DEFAULT_OCTAVE: i32 = 3;
pub static DEFAULT_VELOCITY: f32 = 0.8;

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

static NOTE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([CDEFGAB])(')?(-2|-1|0|1|2|3|4|5|6|7)?([a-z])?(@\d+)?(:\d+)?(;\d+)?$").unwrap()
});

fn parse_note_expr(def: &str) -> Option<Note> {
    match parse_note(def) {
        Some(tup) => Some(tup.0),
        None => None,
    }
}

fn parse_note_tie(def: &str) -> Option<(Note, u32)> {
    match parse_note(def) {
        Some(tup) => Some((tup.0, tup.1)),
        None => None,
    }
}

fn parse_note_repeat(def: &str) -> Option<(Note, u32)> {
    match parse_note(def) {
        Some(tup) => Some((tup.0, tup.2)),
        None => None,
    }
}

fn parse_note_repeat_grouped(def: &str) -> Option<(Note, u32)> {
    match parse_note(def) {
        Some(tup) => Some((tup.0, tup.3)),
        None => None,
    }
}

fn parse_note(def: &str) -> Option<(Note, u32, u32, u32)> {
    let caps = NOTE_REGEX.captures(def).unwrap();
    assert_eq!(caps.len(), 8);
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
    let mut ties: u32 = 1;
    if let Some(matched) = caps.get(5) {
        ties = matched.as_str()[1..].parse().unwrap();
    }
    let mut repeats_no_grouping: u32 = 1;
    if let Some(matched) = caps.get(6) {
        repeats_no_grouping = matched.as_str()[1..].parse().unwrap();
    }
    let mut repeats_grouped: u32 = 1;
    if let Some(matched) = caps.get(7) {
        repeats_grouped = matched.as_str()[1..].parse().unwrap();
    }
    Some((
        Note {
            note_num: note_num as u8,
            velocity: velocity,
            // Duration is specified as ratio relative to the containing event's duration.
            // Event duration is really what determines the rhythm of the overall pattern.
            dur: Dur::new(1, 2),
        },
        ties,
        repeats_no_grouping,
        repeats_grouped,
    ))
}

fn parse_rest_tie(def: &str) -> Option<u32> {
    let ties: u32 = def[2..].parse().unwrap();
    Some(ties)
}

#[derive(Clone, Debug, Logos, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum Token {
    #[token("<")]
    AlternationStart,
    #[token(">")]
    AlternationEnd,
    #[token("[")]
    GroupStart,
    #[token("]")]
    GroupEnd,
    #[regex(r"[CDEFGAB](')?(-2|-1|0|1|2|3|4|5|6|7)?([a-z])?:(\d+)", |lex| parse_note_repeat(lex.slice()))]
    NoteRepeat((Note, u32)),
    #[regex(r"[CDEFGAB](')?(-2|-1|0|1|2|3|4|5|6|7)?([a-z])?;(\d+)", |lex| parse_note_repeat_grouped(lex.slice()))]
    NoteRepeatGrouped((Note, u32)),
    #[regex(r"[CDEFGAB](')?(-2|-1|0|1|2|3|4|5|6|7)?([a-z])?@(\d+)", |lex| parse_note_tie(lex.slice()))]
    NoteTie((Note, u32)),
    #[regex(r"[CDEFGAB](')?(-2|-1|0|1|2|3|4|5|6|7)?([a-z])?", |lex| parse_note_expr(lex.slice()))]
    NoteExpr(Note),
    #[regex(r"\.@(\d+)", |lex| parse_rest_tie(lex.slice()))]
    RestTie(u32),
    #[token(".")]
    Rest,
    #[token("_")]
    Tie,
}

#[cfg(test)]
mod test {
    use crate::dur::Dur;
    use crate::lex::{
        get_velocity, parse_note_expr, parse_note_tie, parse_rest_tie, DEFAULT_VELOCITY, NOTE_REGEX,
    };
    use crate::pattern::Note;

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
    fn test_note_regex() {
        let caps = NOTE_REGEX.captures("C'").unwrap();
        assert_eq!(caps.len(), 8);
        assert_eq!(&caps[1], "C");
        assert_eq!(&caps[2], "'");

        let caps = NOTE_REGEX.captures("C3").unwrap();
        assert_eq!(caps.len(), 8);
        assert_eq!(&caps[1], "C");
        assert_eq!(&caps[3], "3");

        let caps = NOTE_REGEX.captures("Cx").unwrap();
        assert_eq!(caps.len(), 8);
        assert_eq!(&caps[1], "C");
        assert_eq!(&caps[4], "x");

        let caps = NOTE_REGEX.captures("C'3").unwrap();
        assert_eq!(caps.len(), 8);
        assert_eq!(&caps[1], "C");
        assert_eq!(&caps[2], "'");
        assert_eq!(&caps[3], "3");

        let caps = NOTE_REGEX.captures("C'x").unwrap();
        assert_eq!(caps.len(), 8);
        assert_eq!(&caps[1], "C");
        assert_eq!(&caps[2], "'");
        assert_eq!(&caps[4], "x");

        let caps = NOTE_REGEX.captures("C'3x").unwrap();
        assert_eq!(caps.len(), 8);
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
    fn test_parse_note_tie() {
        let note = parse_note_tie("C@3");
        assert_eq!(
            note,
            Some((
                Note {
                    note_num: 60,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                },
                3
            ))
        );

        let note = parse_note_tie("C'@3");
        assert_eq!(
            note,
            Some((
                Note {
                    note_num: 61,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                },
                3
            ))
        );

        let note = parse_note_tie("E1@3");
        assert_eq!(
            note,
            Some((
                Note {
                    note_num: 40,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                },
                3
            ))
        );

        let note = parse_note_tie("D'2@3");
        assert_eq!(
            note,
            Some((
                Note {
                    note_num: 51,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                },
                3
            ))
        );
    }

    #[test]
    fn test_parse_rest_tie() {
        let tok = parse_rest_tie(".@3");
        assert_eq!(tok, Some(3));
    }
}
