use crate::dur::Dur;
use crate::pattern::{Event, Note, Pattern};
use logos::{Lexer, Logos};
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

static NOTE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([CDEFGAB])(')?(-2|-1|0|1|2|3|4|5|6|7)?([a-z])?$").unwrap());

fn note_callback(lex: &mut Lexer<Token>) -> Option<Note> {
    let caps = NOTE_REGEX.captures(lex.slice()).unwrap();
    assert_eq!(caps.len(), 5);
    let mut note_num = get_pitch_class(caps[1].chars().next().unwrap());
    if let Some(sharp) = caps.get(2) {
        note_num += 1
    }
    Some(Note {
        note_num: note_num as u8,
        velocity: DEFAULT_VELOCITY,
        dur: Dur::new(1, 2),
    })
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
enum Token {
    #[token("[")]
    GroupStart,
    #[token("]")]
    GroupEnd,
    #[regex(r"[CDEFGAB](')?(-2|-1|0|1|2|3|4|5|6|7)?([a-z])?", note_callback)]
    NoteExpr(Note),
}

enum PatternElement {
    Note { note_num: i32, velocity: f32 },
    Group(Vec<PatternElement>),
}

fn pat(def: &str) -> Result<Pattern, ParseError> {
    // for c in "abcdefghijklmnopqrstuvwxyz".chars() {
    //     println!("{:?}.to_digit() {:?}", c, c as u32);
    // }
    let length_bars = Dur::new(1, 1);
    Ok(Pattern {
        channel: None,
        events: get_events(def, length_bars),
        length_bars: Some(length_bars),
    })
}

fn get_events(def: &str, length_bars: Dur) -> Vec<Event> {
    let _group = get_groups(def, length_bars);
    vec![]
}

fn get_groups(def: &str, length_bars: Dur) -> PatternElement {
    let _lex: Vec<Token> = Token::lexer(def).map(|res| res.unwrap()).collect();
    let _num_elements = _lex.len();
    // assert!(_num_elements >= 2);
    // assert!(_lex[0] == Token::GroupStart);
    // assert!(_lex[_num_elements - 1] == Token::GroupEnd);
    let _each_dur = length_bars.div_int(_num_elements as i64);
    println!("each_dur is {:?}", _each_dur);
    return PatternElement::Group(vec![]);
}

// fn get_groups_r(def: &str, length_bars: Dur, group: PatternElement) -> PatternElement {
//     let _lex: Vec<Token> = Token::lexer(def).map(|res| res.unwrap()).collect();
//     let _num_elements = _lex.len();
//     let _each_dur = length_bars.div_int(_num_elements as i64);
//     println!("each_dur is {:?}", _each_dur);
//     for tok in _lex {
//         println!("{:?}", tok);
//     }
//     return PatternElement::Group(vec![]);
// }

#[cfg(test)]
mod tests {
    use crate::dur::BAR;
    use crate::mini_notation::{pat, NOTE_REGEX};
    use crate::pattern::Pattern;

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
    fn test_pattern_single_note() {
        assert_eq!(
            pat("[Cx]"),
            Ok(Pattern {
                channel: None,
                length_bars: Some(BAR),
                events: vec![],
            }),
        );
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
}
