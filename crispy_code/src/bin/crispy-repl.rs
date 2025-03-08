use crispy_code::scripting::setup_engine;
use env_logger::Env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(Env::default());

    // Initialize scripting engine
    let engine = setup_engine();

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
