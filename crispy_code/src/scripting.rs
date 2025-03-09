use crate::dsl::notes;
use crate::dur::Dur;
use crate::pattern::NamedPattern;
use crate::plugin::{clear, clearall, start, stop, stopall};
use rhai::Engine;

pub fn setup_engine() -> Engine {
    let mut engine = Engine::new();

    engine
        .register_type_with_name::<Dur>("Dur")
        .register_fn("dur", |n, d| Dur::new(n, d));

    engine
        .register_type_with_name::<NamedPattern>("NamedPattern")
        .register_fn("named", NamedPattern::named)
        .register_fn("reverse", NamedPattern::reverse)
        .register_fn("stretch", NamedPattern::stretch);

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
            engine.eval::<()>(r#"start(notes("[C]").named("polysynth"))"#),
            Ok(_)
        ));

        // Won't error even if the dsl is invalid
        assert!(matches!(
            engine.eval::<()>(r#"start(notes("C]").named("polysynth"))"#),
            Ok(_)
        ));
    }
}
