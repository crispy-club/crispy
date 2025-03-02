use crispy_code::dsl::notes;
use crispy_code::pattern::NamedPattern;
use crispy_code::plugin::play;
use std::error::Error;
use std::io;
use tokio;

fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer = String::new();
    let _input = io::stdin().read_line(&mut buffer)?;
    let pattern = notes(buffer.as_str().trim())?;

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            play(NamedPattern {
                name: String::from("foo"),
                channel: 1,
                length_bars: pattern.length_bars,
                events: pattern.events,
            })
            .await
        })
}
