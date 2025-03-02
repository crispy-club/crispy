use crate::dur::Dur;
use crate::lex::Token;
use crate::parse::{Element, ParseError, Parser};
use crate::pattern::{Event, EventType, Pattern};
use logos::Logos;

// Covered by integration tests
#[allow(dead_code)]
pub fn notes(def: &str) -> Result<Pattern, ParseError> {
    let len_bars = Dur::new(1, 1);
    let events = get_events(def, len_bars)?;
    Ok(Pattern {
        channel: 1,
        events: events,
        length_bars: len_bars,
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
            Token::RestRepeat(repeats) => {
                for _ in 0..repeats {
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
    transform_r(&root, len_bars, &mut events);
    events
}

fn transform_r<'source>(root: &Element, len: Dur, events: &mut Vec<Event>) {
    match root {
        Element::Note(note) => events.push(Event {
            action: EventType::NoteEvent(*note),
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
        Element::Alternation((anchor_element, alt_elements)) => {
            // let alt_len = alt_elements.len();
            // let each_dur = len.div_int(2);
            transform_r(
                &Element::Group(expand_alt(anchor_element, alt_elements.to_vec())),
                len,
                events,
            );
        }
    }
}

fn expand_alt<'source>(anchor: &Element, elements: Vec<Element>) -> Vec<Element> {
    let mut expanded_elements: Vec<Element> = vec![];
    for elem in elements {
        match elem {
            Element::Alternation((alt_anchor, alt_elems)) => {
                let sub_elems = expand_alt(&alt_anchor, alt_elems.to_vec());
                expanded_elements.extend(expand_alt(anchor, sub_elems));
            }
            _ => {
                expanded_elements.push(anchor.clone());
                expanded_elements.push(elem);
            }
        }
    }
    expanded_elements
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
    use crate::dsl::{expand_alt, get_root_elem, Element};
    use crate::dur::Dur;
    use crate::lex::DEFAULT_VELOCITY;
    use crate::pattern::Note;
    use pretty_assertions::assert_eq;

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

    #[test]
    fn test_expand_alt() {
        let actual = expand_alt(
            &Element::Note(Note {
                note_num: 60,
                velocity: DEFAULT_VELOCITY,
                dur: Dur::new(1, 2),
            }),
            vec![
                Element::Note(Note {
                    note_num: 61,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
                Element::Note(Note {
                    note_num: 62,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
            ],
        );
        assert_eq!(
            actual,
            vec![
                Element::Note(Note {
                    note_num: 60,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
                Element::Note(Note {
                    note_num: 61,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
                Element::Note(Note {
                    note_num: 60,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
                Element::Note(Note {
                    note_num: 62,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
            ]
        );
    }

    #[test]
    fn test_expand_alt_nested() {
        let actual = expand_alt(
            &Element::Note(Note {
                note_num: 59,
                velocity: DEFAULT_VELOCITY,
                dur: Dur::new(1, 2),
            }),
            vec![Element::Alternation((
                Box::new(Element::Note(Note {
                    note_num: 60,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                })),
                vec![
                    Element::Note(Note {
                        note_num: 61,
                        velocity: DEFAULT_VELOCITY,
                        dur: Dur::new(1, 2),
                    }),
                    Element::Note(Note {
                        note_num: 62,
                        velocity: DEFAULT_VELOCITY,
                        dur: Dur::new(1, 2),
                    }),
                ],
            ))],
        );
        assert_eq!(
            actual,
            vec![
                Element::Note(Note {
                    note_num: 59,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
                Element::Note(Note {
                    note_num: 60,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
                Element::Note(Note {
                    note_num: 59,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
                Element::Note(Note {
                    note_num: 61,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
                Element::Note(Note {
                    note_num: 59,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
                Element::Note(Note {
                    note_num: 60,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
                Element::Note(Note {
                    note_num: 59,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
                Element::Note(Note {
                    note_num: 62,
                    velocity: DEFAULT_VELOCITY,
                    dur: Dur::new(1, 2),
                }),
            ]
        );
    }
}
