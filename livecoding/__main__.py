import sys

import click
from attrs import define

from livecoding.base_types import Duration, Event, Note, Pattern
from livecoding.grammar import lark_ebnf
from livecoding.notes import NoteNumbers
from livecoding.plugin import note_pattern, play, stop


@define
class Melody:
    def parse(self, line: str) -> Pattern:
        name, definition = line.strip().split("=")
        return note_pattern(name.strip(), definition)


@define
class Perc:
    def parse_event(self, word: str, note_num: int) -> Event:
        if word == ".":
            return Event(action="Rest", dur_frac=Duration(num=1, den=16))
        elif word == "X":
            return Event(
                action=Note(
                    Note.Params(
                        note_num=note_num,
                        velocity=0.9,
                        dur_ms=20,
                    ),
                ),
                dur_frac=Duration(num=1, den=16),
            )
        elif word == "x":
            return Event(
                action=Note(
                    Note.Params(
                        note_num=note_num,
                        velocity=0.4,
                        dur_ms=20,
                    ),
                ),
                dur_frac=Duration(num=1, den=16),
            )
        else:
            raise ValueError(f"unsupported notation: {word}")

    def parse(self, line: str) -> Pattern:
        note_str, events_str = line.strip().split("=")
        note_num = NoteNumbers[note_str.strip()]
        if len(events_str.strip()) == 0:
            return Pattern(name="", length_bars=Duration(0, 1), events=[])
        events = [
            self.parse_event(char, note_num)
            for char in events_str.strip()
            if not char.isspace()
        ]
        return Pattern(
            name=note_str.strip(),
            events=events,
            length_bars=Duration(len(events), 16),
        )


@click.group
def cli() -> None:
    pass


@cli.command()
def ebnf() -> None:
    print(lark_ebnf())


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
    perc = Perc()
    for line in map(lambda ln: ln.strip(), sys.stdin):
        pattern = perc.parse(line)
        if len(pattern.events) == 0:
            continue
        play(pattern)


@cli.command()
def melody() -> None:
    melody = Melody()
    for line in map(lambda ln: ln.strip(), sys.stdin):
        pattern = melody.parse(line)
        if len(pattern.events) == 0:
            continue
        play(pattern)


if __name__ == "__main__":
    cli()
