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
