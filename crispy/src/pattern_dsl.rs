use crate::dur::Dur;
use crate::lex::Token;
use crate::pattern::{Event, EventType, Note, Pattern};
use logos::Logos;

#[derive(Debug, Clone, PartialEq)]
enum ParseError {
    HalfOpenGroup,
}

#[derive(Clone, Debug, PartialEq)]
enum Pelem {
    Group(Vec<Pelem>),
    Note(Note),
    Rest,
}

#[allow(dead_code)]
struct ParsingStart;

struct ParsingPattern;

struct ParsingEnd {
    elems: Vec<Pelem>,
}

struct Parser<State> {
    state: State,
}

impl Parser<ParsingStart> {
    fn new() -> Parser<ParsingPattern> {
        Parser {
            state: ParsingPattern,
        }
    }

    fn new_group() -> Parser<ParsingGroup> {
        Parser {
            state: ParsingGroup,
        }
    }
}

impl Parser<ParsingPattern> {
    fn parse(&mut self, tokens: Vec<Token>) -> Parser<ParsingEnd> {
        let mut elems: Vec<Pelem> = vec![];

        if tokens.len() == 0 {
            return Parser {
                state: ParsingEnd { elems: elems },
            };
        }
        let mut idx = 0;

        while idx < tokens.len() {
            match tokens[idx] {
                Token::GroupStart => {
                    let group = Parser::new_group();
                    let parsed = group.parse((&tokens[(idx + 1)..]).to_vec());
                    let group_elements = parsed.get_elements();
                    let tokens_consumed = parsed.get_tokens_consumed();
                    elems.extend(group_elements);
                    idx += tokens_consumed;
                    continue;
                }
                Token::GroupEnd => {
                    panic!("] without a matching [");
                }
                Token::NoteExpr(note) => {
                    elems.push(Pelem::Note(note));
                }
                Token::Rest => {
                    elems.push(Pelem::Rest);
                }
            }
            idx += 1
        }
        Parser {
            state: ParsingEnd { elems: elems },
        }
    }
}

impl Parser<ParsingEnd> {
    fn get_elements(&self) -> Vec<Pelem> {
        self.state.elems.clone()
    }
}

struct ParsingGroup;

struct GroupEnd {
    elems: Vec<Pelem>,
    tokens_consumed: usize,
}

impl Parser<ParsingGroup> {
    fn parse(&self, tokens: Vec<Token>) -> Parser<GroupEnd> {
        let mut elems: Vec<Pelem> = vec![];

        if tokens.len() == 0 {
            return Parser {
                state: GroupEnd {
                    elems: elems,
                    tokens_consumed: 0,
                },
            };
        }
        let mut idx = 0;

        while idx < tokens.len() {
            match tokens[idx] {
                Token::GroupStart => {
                    let group = Parser::new_group();
                    let parsed = group.parse((&tokens[(idx + 1)..]).to_vec());
                    let group_elements = parsed.get_elements();
                    let tokens_consumed = parsed.get_tokens_consumed();
                    elems.extend(group_elements);
                    idx += tokens_consumed;
                    continue;
                }
                Token::GroupEnd => {
                    return Parser {
                        state: GroupEnd {
                            elems: elems,
                            tokens_consumed: idx + 1,
                        },
                    };
                }
                Token::NoteExpr(note) => {
                    elems.push(Pelem::Note(note));
                }
                Token::Rest => {
                    elems.push(Pelem::Rest);
                }
            }
            idx += 1
        }
        Parser {
            state: GroupEnd {
                elems: elems,
                tokens_consumed: idx,
            },
        }
    }
}

impl Parser<GroupEnd> {
    fn get_elements(&self) -> Vec<Pelem> {
        self.state.elems.clone()
    }

    fn get_tokens_consumed(&self) -> usize {
        self.state.tokens_consumed
    }
}

fn get_events(def: &str, len_bars: Dur) -> Result<Vec<Event>, ParseError> {
    let root_elem = get_root_elem(def)?;
    Ok(transform(root_elem, len_bars))
}

fn get_root_elem(def: &str) -> Result<Pelem, ParseError> {
    let tokens: Vec<Token> = Token::lexer(def).map(|res| res.unwrap()).collect();
    let mut parser = Parser::new();
    let parsed = parser.parse(tokens);
    Ok(Pelem::Group(parsed.get_elements()))
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
        Pelem::Rest => events.push(Event {
            action: EventType::Rest,
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
    let events = get_events(def, len_bars)?;
    Ok(Pattern {
        channel: None,
        events: events,
        length_bars: Some(len_bars),
    })
}

#[cfg(test)]
mod tests {
    use crate::dur::{Dur, BAR};
    use crate::lex::DEFAULT_VELOCITY;
    use crate::pattern::{Event, EventType, Note, Pattern};
    use crate::pattern_dsl::{get_root_elem, pat, ParseError, Pelem};
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
    fn test_pattern_group_missing_right_delimiter() {
        assert_eq!(pat("["), Err(ParseError::HalfOpenGroup));
    }

    #[test]
    fn test_pattern_group_missing_left_delimiter() {
        assert_eq!(pat("]"), Err(ParseError::HalfOpenGroup));
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
    fn test_pattern_with_subpattern_first() {
        let actual = pat("[[D'g G4u] Cx]");
        let expect = Pattern {
            channel: None,
            length_bars: Some(BAR),
            events: vec![
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
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 60,
                        velocity: 0.89,
                        dur: Dur::new(1, 2),
                    }),
                    dur: Dur::new(1, 2),
                },
            ],
        };
        assert_eq!(actual, Ok(expect));
    }

    #[test]
    fn test_pattern_with_subpattern_last() {
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
    fn test_get_root_elem() {
        let elem = get_root_elem("[C]");
        assert_eq!(
            elem,
            Ok(Pelem::Group(vec![Pelem::Note(Note {
                note_num: 60,
                velocity: DEFAULT_VELOCITY,
                dur: Dur::new(1, 2),
            })]))
        );
    }

    #[test]
    fn test_get_root_elem_subgroup() {
        let elem = get_root_elem("C [D E]");
        assert_eq!(
            elem,
            Ok(Pelem::Group(vec![
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
            ]))
        );
    }
}
