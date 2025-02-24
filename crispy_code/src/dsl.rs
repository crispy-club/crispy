use crate::dur::Dur;
use crate::lex::Token;
use crate::parse::{Element, ParseError, Parser};
use crate::pattern::{Event, EventType, Pattern};
use logos::Logos;

fn pat(def: &str) -> Result<Pattern, ParseError> {
    let len_bars = Dur::new(1, 1);
    let events = get_events(def, len_bars)?;
    Ok(Pattern {
        channel: None,
        events: events,
        length_bars: Some(len_bars),
    })
}

fn get_events(def: &str, len_bars: Dur) -> Result<Vec<Event>, ParseError> {
    let root_elem = get_root_elem(def)?;
    Ok(transform(root_elem, len_bars))
}

fn get_root_elem(def: &str) -> Result<Element, ParseError> {
    let tokens: Vec<Token> = Token::lexer(def).map(|res| res.unwrap()).collect();
    let mut parser = Parser::new();
    let parsed = parser.parse(desugar(tokens))?;
    let elements = parsed.get_elements();
    if elements.len() == 1 && matches!(elements[0], Element::Group(_)) {
        return Ok(elements[0].clone());
    }
    Ok(Element::Group(elements))
}

fn desugar(tokens: Vec<Token>) -> Vec<Token> {
    let len = tokens.len();
    let mut res = Vec::with_capacity(len);
    for tok in tokens {
        match tok {
            Token::NoteTie((note, ties)) => {
                res.push(Token::NoteExpr(note));
                assert!(ties > 2);
                for _ in 0..(ties - 1) {
                    res.push(Token::Tie);
                }
            }
            Token::RestTie(ties) => {
                for _ in 0..ties {
                    res.push(Token::Rest);
                }
            }
            Token::NoteRepeat((note, repeats)) => {
                for _ in 0..repeats {
                    res.push(Token::NoteExpr(note));
                }
            }
            Token::NoteRepeatGrouped((note, repeats)) => {
                res.push(Token::GroupStart);
                for _ in 0..repeats {
                    res.push(Token::NoteExpr(note));
                }
                res.push(Token::GroupEnd);
            }
            any => res.push(any),
        }
    }
    res
}

fn transform<'source>(root: Element, len_bars: Dur) -> Vec<Event> {
    let mut events: Vec<Event> = vec![];
    transform_r(root, len_bars, &mut events);
    events
}

fn transform_r<'source>(root: Element, len: Dur, events: &mut Vec<Event>) {
    match root {
        Element::Note(note) => events.push(Event {
            action: EventType::NoteEvent(note),
            dur: len,
        }),
        Element::Rest => events.push(Event {
            action: EventType::Rest,
            dur: len,
        }),
        Element::Tie => {
            handle_tie(len, events);
        }
        Element::Group(elems) => {
            if elems.len() == 0 {
                return;
            }
            let num_elems = elems.len();
            let each_dur = len.div_int(num_elems as i64);
            for elem in elems {
                transform_r(elem, each_dur, events);
            }
        }
        Element::Alternation(_) => {}
    }
}

fn handle_tie<'source>(len: Dur, events: &mut Vec<Event>) {
    // Extend duration of previous event.
    // If the previous event was a note, we also extend the note's duration.
    assert!(events.len() > 0);
    let num_events = events.len();
    let prev = &events[num_events - 1];
    events[num_events - 1] = Event {
        action: prev.action.clone(),
        dur: prev.dur + len,
    };
}

#[cfg(test)]
mod tests {
    use crate::dsl::{get_root_elem, pat, Element, ParseError};
    use crate::dur::{Dur, BAR};
    use crate::lex::DEFAULT_VELOCITY;
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
    fn test_pattern_missing_group_delimiter() {
        assert_eq!(pat("["), Err(ParseError::MissingGroupDelimiter));
        assert_eq!(pat("]"), Err(ParseError::MissingGroupDelimiter));
        assert_eq!(pat("] C3"), Err(ParseError::MissingGroupDelimiter));
        assert_eq!(pat("C3 ]"), Err(ParseError::MissingGroupDelimiter));
        assert_eq!(pat("[ C3"), Err(ParseError::MissingGroupDelimiter));
        assert_eq!(pat("C3 ["), Err(ParseError::MissingGroupDelimiter));
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
    fn test_pattern_single_note_plus_rest() {
        assert_eq!(
            pat("[Cx .]"),
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
                        action: EventType::Rest,
                        dur: Dur::new(1, 2),
                    },
                ],
            }),
        );

        assert_eq!(
            pat("Cx ."),
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
                        action: EventType::Rest,
                        dur: Dur::new(1, 2),
                    },
                ],
            }),
        );
    }

    #[test]
    fn test_pattern_with_ties() {
        assert_eq!(
            pat("[Cx Gp _ _]"),
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
                        dur: Dur::new(1, 4),
                    },
                    Event {
                        action: EventType::NoteEvent(Note {
                            note_num: 67,
                            velocity: 0.59,
                            dur: Dur::new(1, 2),
                        }),
                        dur: Dur::new(3, 4),
                    },
                ],
            }),
        );

        assert_eq!(
            pat("[Cx Gp@3]"),
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
                        dur: Dur::new(1, 4),
                    },
                    Event {
                        action: EventType::NoteEvent(Note {
                            note_num: 67,
                            velocity: 0.59,
                            dur: Dur::new(1, 2),
                        }),
                        dur: Dur::new(3, 4),
                    },
                ],
            }),
        );

        assert_eq!(
            pat("[Cx .@2 Eo]"),
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
                        dur: Dur::new(1, 4),
                    },
                    Event {
                        action: EventType::Rest,
                        dur: Dur::new(1, 4),
                    },
                    Event {
                        action: EventType::Rest,
                        dur: Dur::new(1, 4),
                    },
                    Event {
                        action: EventType::NoteEvent(Note {
                            note_num: 64,
                            velocity: 0.56,
                            dur: Dur::new(1, 2),
                        }),
                        dur: Dur::new(1, 4),
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
    fn test_pattern_with_nongrouping_repeat() {
        let actual = pat("[Cx D'g:2 G4u]");
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
                    dur: Dur::new(1, 4),
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
    fn test_pattern_with_grouping_repeat() {
        let actual = pat("[Cx D'g;2 G4u]");
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
                    dur: Dur::new(1, 3),
                },
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 63,
                        velocity: 0.26,
                        dur: Dur::new(1, 2),
                    }),
                    dur: Dur::new(1, 6),
                },
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 63,
                        velocity: 0.26,
                        dur: Dur::new(1, 2),
                    }),
                    dur: Dur::new(1, 6),
                },
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 79,
                        velocity: 0.78,
                        dur: Dur::new(1, 2),
                    }),
                    dur: Dur::new(1, 3),
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
            Ok(Element::Group(vec![Element::Note(Note {
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
            Ok(Element::Group(vec![
                Element::Note(Note {
                    note_num: 60,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
                Element::Group(vec![
                    Element::Note(Note {
                        note_num: 62,
                        velocity: DEFAULT_VELOCITY,
                        dur: Dur::new(1, 2),
                    }),
                    Element::Note(Note {
                        note_num: 64,
                        velocity: DEFAULT_VELOCITY,
                        dur: Dur::new(1, 2),
                    }),
                ]),
            ]))
        );
    }
}
