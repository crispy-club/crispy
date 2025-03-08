use crispy_code::dsl::notes;
use crispy_code::dur::Dur;
use crispy_code::pattern::NamedPattern;
use crispy_code::plugin::play;
use env_logger::Env;
use rhai::{Dynamic, Engine, EvalAltResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(Env::default());

    // Initialize scripting engine
    let mut engine = Engine::new();

    engine.build_type::<NamedPattern>();

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
    engine.register_fn(
        "play",
        |np: NamedPattern| -> Result<(), Box<EvalAltResult>> {
            play(np).map_err(|e| {
                Box::new(EvalAltResult::ErrorRuntime(
                    Dynamic::from(e.to_string()),
                    rhai::Position::NONE,
                ))
            })
        },
    );

    // REPL line editor setup
    let mut rl = rustyline::DefaultEditor::new()?;

    'main_loop: loop {
        let cmd = rl.readline("$ ")?;

        match cmd.as_str() {
            // match rl.readline("$ ") {
            "q" => {
                break 'main_loop;
            }
            code => {
                // println!("{:?}", code);
                if let Err(err) = engine.run(&code) {
                    return Err(Box::new(err));
                }
            }
        }
    }

    Ok(())
}
