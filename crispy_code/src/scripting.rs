use crate::dsl::notes;
use crate::dur::Dur;
use crate::http_commands::{clear, clearall, start, stop, stopall};
use crate::pattern::NamedPattern;
use crate::scales::{scale, scali, Scales};
use rhai::{Array, Dynamic, Engine};

pub fn setup_engine() -> Engine {
    let mut engine = Engine::new();

    engine
        .register_type_with_name::<Dur>("Dur")
        .register_fn("dur", |n, d| Dur::new(n, d));

    engine
        .register_type_with_name::<NamedPattern>("NamedPattern")
        .register_fn("named", NamedPattern::named)
        .register_fn("note", NamedPattern::note)
        .register_fn("reverse", NamedPattern::reverse)
        .register_fn("len", NamedPattern::len)
        .register_fn("trans", NamedPattern::trans);

    engine.register_fn("notes", |expr: &str| -> NamedPattern {
        match notes(expr) {
            Err(err) => {
                eprintln!("error with pattern: {}", err);
                NamedPattern {
                    channel: 1,
                    events: vec![],
                    name: String::from("erased what you were doing"),
                    length_bars: Dur::new(1, 1),
                }
            }
            Ok(pat) => pat,
        }
    });

    register_commands(&mut engine);
    register_scales(&mut engine);

    engine
}

fn register_commands(engine: &mut Engine) {
    engine.register_fn("start", |np: NamedPattern| {
        if let Err(err) = start(np) {
            eprintln!("error starting pattern: {}", err);
        }
    });
    engine.register_fn("stop", |np: NamedPattern| {
        if let Err(err) = stop(np) {
            eprintln!("error stopping pattern: {}", err);
        }
    });
    engine.register_fn("stopall", || {
        if let Err(err) = stopall() {
            eprintln!("error stopping all patterns: {}", err);
        }
    });
    engine.register_fn("clear", |np: NamedPattern| {
        if let Err(err) = clear(np) {
            eprintln!("error clearing pattern: {}", err);
        }
    });
    engine.register_fn("clearall", || {
        if let Err(err) = clearall() {
            eprintln!("error clearing all patterns: {}", err);
        }
    });
}

fn pitch_classes_to_array(pitch_classes: Vec<u8>) -> Array {
    pitch_classes
        .into_iter()
        .map(|x| Dynamic::from(x))
        .collect()
}

fn register_scales(engine: &mut Engine) {
    for (scale_name, pitch_classes) in Scales.iter() {
        engine.register_global_module(rhai::Shared::new({
            let mut module = rhai::Module::new();
            module.set_var(*scale_name, pitch_classes_to_array(pitch_classes.clone()));
            module
        }));
    }
    engine.register_fn("scale", |key, def, pitch_classes: Array| {
        // Expect pitch_classes will be Vec<Dynamic> and we need to convert to Vec<u8>
        let result = scale(
            key,
            def,
            pitch_classes
                .into_iter()
                .map(|x| x.try_cast::<u8>().expect("should be u8"))
                .collect::<Vec<u8>>(),
        );
        result.expect("error creating pattern with scale")
    });
    engine.register_fn("scali", |key, def, pitch_classes: Array, idx: Array| {
        // Expect pitch_classes will be Vec<Dynamic> and we need to convert to Vec<u8>
        let result = scali(
            key,
            def,
            pitch_classes
                .into_iter()
                .map(|x| x.try_cast::<u8>().expect("cast Dynamic to u8"))
                .collect::<Vec<u8>>(),
            idx.into_iter()
                .map(|x| {
                    usize::try_from(x.try_cast::<i64>().expect("cast Dynamic to i64"))
                        .expect("convert i64 to usize")
                })
                .collect::<Vec<usize>>(),
        );
        result.expect("error creating pattern with scale")
    });
}

#[cfg(test)]
mod tests {
    use crate::dur::Dur;
    use crate::pattern::{Event, EventType, NamedPattern, Note};
    use crate::scripting::setup_engine;
    use rhai::Dynamic;

    #[test]
    fn test_engine() {
        let engine = setup_engine();

        // Passes a basic pattern
        assert!(matches!(
            engine.eval::<()>(r#"start(notes("[C]").named("polysynth"))"#),
            Ok(_)
        ));

        // Won't error even if the dsl is invalid
        assert!(matches!(
            engine.eval::<()>(r#"start(notes("C]").named("polysynth"))"#),
            Ok(_)
        ));
    }

    #[test]
    fn test_scripting_scales_are_global_variables() {
        let engine = setup_engine();

        let result = engine
            .eval_expression::<Vec<Dynamic>>("acoustic")
            .unwrap()
            .into_iter()
            .map(|d| d.try_cast::<u8>().expect("should be u8"))
            .collect::<Vec<u8>>();

        // Passes a basic pattern
        assert_eq!(
            result,
            vec![0, 2, 4, 6, 7, 9, 10]
                .into_iter()
                .map(|x| x as u8)
                .collect::<Vec<u8>>()
        );
    }

    #[test]
    fn test_scripting_create_named_pattern_from_scale() {
        let engine = setup_engine();

        let result = engine
            .eval_expression::<NamedPattern>(r#"scale("F'", "x t d o", persian)"#)
            .unwrap()
            .named("foo");

        // Passes a basic pattern
        assert_eq!(
            result,
            NamedPattern {
                channel: 1,
                // ("persian", vec![0, 1, 4, 5, 6, 8, 11]),
                events: vec![(66, 0.89, 4), (67, 0.74, 4), (70, 0.15, 4), (71, 0.56, 4),]
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
    fn test_scripting_create_named_pattern_from_scali() {
        let engine = setup_engine();

        let result = engine
            .eval_expression::<NamedPattern>(r#"scali("F'", "x t d o", persian, [2, 4, 5, 1])"#)
            .unwrap()
            .named("foo");

        // Passes a basic pattern
        assert_eq!(
            result,
            NamedPattern {
                channel: 1,
                // ("persian", vec![0, 1, 4, 5, 6, 8, 11]),
                events: vec![(70, 0.89, 4), (72, 0.74, 4), (74, 0.15, 4), (67, 0.56, 4),]
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
