import sys

import click

from livecoding.base_types import Duration, Event, Note, Pattern
from livecoding.grammar import lark_ebnf
from livecoding.plugin import play, stop


NOTES = {
    f"{base_note}{octave}": (octave * 12) + note_idx + 24
    for (note_idx, base_note) in enumerate(
        ("c", "c#", "d", "d#", "e", "f", "f#", "g", "g#", "a", "a#", "b")
    )
    for octave in range(8)
}


def _parse_event(word: str, note_num: int) -> Event:
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


def _parse(line: str) -> Pattern:
    note_str, events_str = line.strip().split("=")
    note_num = NOTES[note_str.strip()]
    if len(events_str.strip()) == 0:
        return Pattern(name="", length_bars=Duration(0, 1), events=[])
    events = [
        _parse_event(char, note_num)
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
def silence() -> None:
    for pattern_name in NOTES:
        stop(pattern_name)


@cli.command()
def start() -> None:
    for line in map(lambda ln: ln.strip(), sys.stdin):
        pattern = _parse(line)
        if len(pattern.events) == 0:
            continue
        play(pattern)


if __name__ == "__main__":
    cli()
