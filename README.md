# crispy

`crispy` is yet another tool for [livecoding](https://en.wikipedia.org/wiki/Live_coding).
It runs as a plugin in whatever host software you like to use.
Both VST3 and CLAP plugin formats are supported.
It is able to trigger sample-accurate MIDI events due to the fact that it is a plugin.
The MIDI events it outputs are controlled by code that is edited in the GUI.
This is very much a work in progress and YMMV.

## Development

Some of the core beliefs embodied in this project

* Code should be refactored frequently to maintain high quality
* Code should be as lightweight as possible in terms of dependencies to make it easy to write unit tests
* Test coverage should be very high
* New code should be thoroughly tested with both unit tests and manual testing

The repo is structured as a [workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
with each package being a separate plugin.

The main livecoding plugin is [crispy_code](https://github.com/crispy-club/crispy/tree/main/crispy_code) aka `CODE`.

We plan to have automated builds eventually that will generate github releases with compiled binaries available
for download. In the meantime the only option to install locally is to build from source
```
cargo xtask bundle crispy_code --release
```

This will output artifacts in the `target/bundled/` directory.
To load the plugin, your host application (e.g. your DAW) will need to be able to access this directory,
and will be different for everyone based on where you've checked out the source code on your machine.

### Testing

#### Generating Coverage Reports

You can generate coverage reports locally with [grcov](https://github.com/mozilla/grcov).

There will be some setup involved.

Necessary env vars are
```
export CARGO_INCREMENTAL=0
export RUSTFLAGS='-Cinstrument-coverage'
export LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw'
```

Install llvm-tools
```
rustup component add llvm-tools-preview
```

Run the tests
```
cargo test --package crispy_code
```

Create the directory for the coverage report
```
mkdir -p target/coverage
```

Then run grcov and output html
```
grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/
```

If all of these steps worked, you should be able to find the coverage report at
target/coverage/html/index.html
