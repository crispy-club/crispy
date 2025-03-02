use crispy_code::dsl::notes;
use crispy_code::pattern::NamedPattern;
use crispy_code::plugin::play;
use env_logger::Env;
use rhai::{Dynamic, Engine, EvalAltResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(Env::default());

    // Initialize scripting engine
    let mut engine = Engine::new();

    engine.build_type::<NamedPattern>();

    engine.register_fn("notes", notes);
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

    let cmd = rl.readline("$ ")?;

    'main_loop: loop {
        match cmd.as_str() {
            // match rl.readline("$ ") {
            "q" => {
                break 'main_loop;
            }
            code => {
                if let Err(err) = engine.run(&code) {
                    return Err(Box::new(err));
                }
            }
        }
    }

    Ok(())
}
