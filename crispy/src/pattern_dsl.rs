use crate::dur::Dur;
use crate::lex::Token;
use crate::pattern::{Event, EventType, Note, Pattern};
use logos::Logos;

#[derive(Debug, Clone, PartialEq)]
struct ParseError;

#[derive(Clone, Debug, PartialEq)]
enum Pelem {
    Rest,
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
        Pelem::Rest => 0,
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
                    Token::Rest => {
                        elems.push(Pelem::Rest);
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
    use crate::lex::DEFAULT_VELOCITY;
    use crate::pattern_dsl::{get_root_elem, pat, Pelem};
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
