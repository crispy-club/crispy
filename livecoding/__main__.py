import sys
from dataclasses import dataclass

import click

from livecoding.base_types import PluginPattern
from livecoding.filters import name
from livecoding.notes import NoteNumbers
from livecoding.pat import pat as _pat
from livecoding.perc import perc as _perc
from livecoding.plugin import play, stop


@dataclass(slots=True)
class Melody:
    def parse(self, line: str) -> PluginPattern:
        _name, definition = line.strip().split("=")
        return _pat(definition) | name(_name.strip())


@click.group
def cli() -> None:
    pass


@cli.command()
@click.option("--name", required=False)
@click.option("--notes", is_flag=True, default=False)
def silence(name: str | None, notes: bool) -> None:
    assert (name is not None and len(name.strip()) > 0) or notes
    if notes:
        for pattern_name in NoteNumbers:
            stop(pattern_name)
    else:
        assert name is not None
        stop(name)


@cli.command()
def perc() -> None:
    play(*_perc(sys.stdin.read()))


@cli.command()
def pat() -> None:
    melody = Melody()
    for line in map(lambda ln: ln.strip(), sys.stdin):
        pattern = melody.parse(line)
        if len(pattern.events) == 0:
            continue
        play(pattern)


if __name__ == "__main__":
    cli()
