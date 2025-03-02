use crispy_code::dsl::notes;
use crispy_code::pattern::NamedPattern;
use crispy_code::plugin::play;
use std::error::Error;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer = String::new();
    let _input = io::stdin().read_line(&mut buffer)?;
    let pattern = notes(buffer.as_str().trim())?;

    play(NamedPattern {
        name: String::from("foo"),
        channel: 1,
        length_bars: pattern.length_bars,
        events: pattern.events,
    })
    .map_err(|e| e.into())
}
