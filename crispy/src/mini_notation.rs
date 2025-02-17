use crate::dur::Dur;
use crate::pattern::{Event, Note, Pattern};
use logos::{Lexer, Logos};

#[derive(Debug, Clone, PartialEq)]
struct ParseError;

fn note_callback(lex: &mut Lexer<Token>) -> Option<Note> {
    Some(Note {
        note_num: 60,
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
    #[regex(r"[CDEFGABC](')?(-2|-1|0|1|2|3|4|5|6|7)?[a-z]?", note_callback)]
    NoteExpr(Note),
}

enum PatternElement {
    Note { note_num: i32, velocity: f32 },
    Group(Vec<PatternElement>),
}

fn pat(def: &str) -> Result<Pattern, ParseError> {
    Ok(Pattern {
        channel: None,
        events: get_events(def),
        length_bars: None,
    })
}

fn get_events(def: &str) -> Vec<Event> {
    let _group = get_groups(def);
    vec![]
}

fn get_groups(def: &str) -> PatternElement {
    let _lex: Vec<Token> = Token::lexer(def).map(|res| res.unwrap()).collect();
    for tok in _lex {
        println!("{:?}", tok);
    }
    return PatternElement::Group(vec![]);
}

static DEFAULT_OCTAVE: i32 = 3;
static DEFAULT_VELOCITY: f32 = 0.8;

#[cfg(test)]
mod tests {
    use crate::mini_notation::pat;
    use crate::pattern::Pattern;

    #[test]
    fn test_pattern_empty() {
        assert_eq!(
            pat("[]"),
            Ok(Pattern {
                channel: None,
                length_bars: None,
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
                length_bars: None,
                events: vec![],
            }),
        );
    }
}
