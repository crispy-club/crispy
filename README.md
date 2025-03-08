# crispy

`crispy` is yet another tool for [livecoding](https://en.wikipedia.org/wiki/Live_coding).
It runs as a plugin in whatever host software you like to use.
Both VST3 and CLAP plugin formats are supported.
It outputs MIDI events that are defined by code.

## Architecture

`crispy` is able to trigger sample-accurate MIDI events due to the fact that it is a plugin.
When you load an instance of `crispy` it will start an HTTP server in a background thread.
The http server is responsible for all the user interaction with the plugin.
The http server shuts down when it receives a shutdown signal from the plugin (when the plugin is deactivated).
The http server also exposes a way to get information about the current state of the plugin.


## Usage

### Build the plugin

```
cargo xtask bundle crispy_code --release
```

CLAP and VST plugins should now be generated in target/bundled/

### Play patterns

Load the plugin into your host software and try this command

```
echo 'C3k [E3k <A4t B4p>] G3k A3k' | cargo run -p crispy_code --bin play
```

The plugin should now be outputting MIDI data which you can then turn into sounds.

### Run a REPL

This command will allow you to play with rhai code in a REPL.
More documentation will be available in the future!

```
cargo run -p crispy_code --bin crispy-repl
```

### Run a script

* Install the `crispy-run` tool

```
cargo install --path crispy_code --bin crispy-run
```

* Run a script

```
crispy-run /path/to/script.rhai
```
