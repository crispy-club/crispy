use crispy_code::dsl::notes;
use crispy_code::dur::{Dur, BAR};
use crispy_code::parse::ParseError;
use crispy_code::pattern::{Event, EventType, Note, Pattern};
use pretty_assertions::assert_eq;

#[test]
fn test_pattern_empty() {
    assert_eq!(
        notes("[]"),
        Ok(Pattern {
            channel: None,
            length_bars: Some(BAR),
            events: vec![],
        }),
    );
}

#[test]
fn test_pattern_missing_group_delimiter() {
    assert_eq!(notes("["), Err(ParseError::MissingGroupDelimiter));
    assert_eq!(notes("]"), Err(ParseError::MissingGroupDelimiter));
    assert_eq!(notes("] C3"), Err(ParseError::MissingGroupDelimiter));
    assert_eq!(notes("C3 ]"), Err(ParseError::MissingGroupDelimiter));
    assert_eq!(notes("[ C3"), Err(ParseError::MissingGroupDelimiter));
    assert_eq!(notes("C3 ["), Err(ParseError::MissingGroupDelimiter));
}

#[test]
fn test_pattern_single_note() {
    assert_eq!(
        notes("[Cx]"),
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
        notes("[Cx D'g]"),
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
fn test_pattern_two_notes_just_velocity() {
    assert_eq!(
        notes("x g"),
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
                        note_num: 60,
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
fn test_pattern_four_notes() {
    assert_eq!(
        notes("C3k E3k G3k A3k"),
        Ok(Pattern {
            channel: None,
            length_bars: Some(BAR),
            events: vec![
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 60,
                        velocity: 0.41,
                        dur: Dur::new(1, 2),
                    }),
                    dur: Dur::new(1, 4),
                },
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 64,
                        velocity: 0.41,
                        dur: Dur::new(1, 2),
                    }),
                    dur: Dur::new(1, 4),
                },
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 67,
                        velocity: 0.41,
                        dur: Dur::new(1, 2),
                    }),
                    dur: Dur::new(1, 4),
                },
                Event {
                    action: EventType::NoteEvent(Note {
                        note_num: 69,
                        velocity: 0.41,
                        dur: Dur::new(1, 2),
                    }),
                    dur: Dur::new(1, 4),
                },
            ],
        })
    );
}

#[test]
fn test_pattern_single_note_plus_rest() {
    assert_eq!(
        notes("[Cx .]"),
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
        notes("Cx ."),
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
fn test_pattern_single_note_plus_rest_tie() {
    assert_eq!(
        notes("[Cx .@3]"),
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
                    action: EventType::Rest,
                    dur: Dur::new(1, 4),
                },
            ],
        }),
    );
}

#[test]
fn test_pattern_single_note_plus_rest_repeat() {
    assert_eq!(
        notes("[Cx .:3]"),
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
                    action: EventType::Rest,
                    dur: Dur::new(1, 4),
                },
            ],
        }),
    );
}

#[test]
fn test_pattern_with_ties() {
    assert_eq!(
        notes("[Cx Gp _ _]"),
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
        notes("[Cx Gp@3]"),
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
        notes("[Cx .@2 Eo]"),
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
    let actual = notes("[[D'g G4u] Cx]");
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
    let actual = notes("[Cx [D'g G4u]]");
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
    let actual = notes("[Cx D'g:2 G4u]");
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
    let actual = notes("[Cx D'g;2 G4u]");
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
fn test_pattern_with_alternation() {
    let actual = notes("[Cx <D'g G4u>]");
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
                    note_num: 60,
                    velocity: 0.89,
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
fn test_pattern_with_nested_alternation() {
    let actual = notes("Cx <D'g <G4u E2l>>");
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
                dur: Dur::new(1, 8),
            },
            Event {
                action: EventType::NoteEvent(Note {
                    note_num: 63,
                    velocity: 0.26,
                    dur: Dur::new(1, 2),
                }),
                dur: Dur::new(1, 8),
            },
            Event {
                action: EventType::NoteEvent(Note {
                    note_num: 60,
                    velocity: 0.89,
                    dur: Dur::new(1, 2),
                }),
                dur: Dur::new(1, 8),
            },
            Event {
                action: EventType::NoteEvent(Note {
                    note_num: 79,
                    velocity: 0.78,
                    dur: Dur::new(1, 2),
                }),
                dur: Dur::new(1, 8),
            },
            Event {
                action: EventType::NoteEvent(Note {
                    note_num: 60,
                    velocity: 0.89,
                    dur: Dur::new(1, 2),
                }),
                dur: Dur::new(1, 8),
            },
            Event {
                action: EventType::NoteEvent(Note {
                    note_num: 63,
                    velocity: 0.26,
                    dur: Dur::new(1, 2),
                }),
                dur: Dur::new(1, 8),
            },
            Event {
                action: EventType::NoteEvent(Note {
                    note_num: 60,
                    velocity: 0.89,
                    dur: Dur::new(1, 2),
                }),
                dur: Dur::new(1, 8),
            },
            Event {
                action: EventType::NoteEvent(Note {
                    note_num: 52,
                    velocity: 0.44,
                    dur: Dur::new(1, 2),
                }),
                dur: Dur::new(1, 8),
            },
        ],
    };
    assert_eq!(actual, Ok(expect));
}

#[test]
fn test_pattern_missing_alternation_delimiter() {
    let actual = notes("Cx <D'g G4u");
    assert_eq!(actual, Err(ParseError::MissingAlternationDelimiter));
    let actual = notes("Cx D'g G4u>");
    assert_eq!(actual, Err(ParseError::MissingAlternationDelimiter));
}

#[test]
fn test_pattern_missing_alternation_anchor() {
    let actual = notes("<D'g G4u>");
    assert_eq!(actual, Err(ParseError::MissingAlternationAnchor));
}
