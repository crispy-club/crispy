use crate::dsl::notes;
use crate::dur::Dur;
use crate::pattern::NamedPattern;
use crate::plugin::play;
use rhai::Engine;

pub fn setup_engine() -> Engine {
    let mut engine = Engine::new();

    engine
        .register_type_with_name::<NamedPattern>("NamedPattern")
        .register_fn("named", NamedPattern::named);

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
    engine.register_fn("play", |np: NamedPattern| {
        if let Err(err) = play(np) {
            eprintln!("error playing pattern: {}", err);
        }
    });

    engine
}

#[cfg(test)]
mod tests {
    use crate::scripting::setup_engine;

    #[test]
    fn test_engine() {
        let engine = setup_engine();

        // Passes a basic pattern
        assert!(matches!(
            engine.eval::<()>(r#"play(notes("[C]").named("polysynth"))"#),
            Ok(_)
        ));

        // Won't error even if the dsl is invalid
        assert!(matches!(
            engine.eval::<()>(r#"play(notes("C]").named("polysynth"))"#),
            Ok(_)
        ));
    }
}
