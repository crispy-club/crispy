use crispy_code::scripting::setup_engine;
use env_logger::Env;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(Env::default());
    let engine = setup_engine();
    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 2, "usage: crispy-run SCRIPT");
    let filename = &args[1];
    if let Err(err) = engine.eval_file::<()>(filename.into()) {
        eprintln!("error running {}: {:?}", filename, err);
    }
    Ok(())
}
