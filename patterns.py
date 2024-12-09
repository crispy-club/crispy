import sys

from typing import Literal, NamedTuple

import click
import jsonplus
import requests


jsonplus.prefer_compat()


NOTES = {
    f"{base_note}{octave}": (octave * 12) + note_idx + 24
    for (note_idx, base_note) in enumerate(
        ("c", "c#", "d", "d#", "e", "f", "f#", "g", "g#", "a", "a#", "b")
    )
    for octave in range(8)
}


class Note(NamedTuple):
    class Event(NamedTuple):
        note_num: int
        velocity: float
        dur_ms: int

    NoteEvent: Event


class FractionalDuration(NamedTuple):
    num: int
    den: int


class Event(NamedTuple):
    action: str | Note
    dur_frac: FractionalDuration


class Pattern(NamedTuple):
    events: list[Event]
    name: str

    @classmethod
    def parse(cls, spec: str) -> "Pattern":
        return _parse(spec)

    def __add__(self, other: "Pattern") -> "Pattern":
        return Pattern(name=name, events=self.events + other.events)

    def start(self) -> None:
        data = jsonplus.dumps({"events": self.events})
        resp = requests.post(
            f"http://127.0.0.1:3000/start/{self.name}",
            headers={"Content-Type": "application/json"},
            data=data,
        )
        resp.raise_for_status()

    def stop(self) -> None:
        resp = requests.post(f"http://127.0.0.1:3000/stop/{self.name}")
        resp.raise_for_status()


class Scene(NamedTuple):
    patterns: dict[str, Pattern]

    @classmethod
    def parse(cls, spec: str) -> "Scene":
        return cls(
            patterns={
                pattern.name: pattern
                for pattern in map(
                    lambda ln: Pattern.parse(ln),
                    filter(
                        lambda ln: len(ln) > 0,
                        map(lambda ln: ln.strip(), spec.split("\n")),
                    ),
                )
            }
        )

    def start(self) -> None:
        for _, pattern in self.patterns.items():
            pattern.start()

    def stop(self) -> None:
        for _, pattern in self.patterns.items():
            pattern.stop()


def _parse_event(word: str, note_num: int) -> Event | None:
    if word == ".":
        return Event(action="Rest", dur_frac=FractionalDuration(num=1, den=16))
    elif word == "X":
        return Event(
            action=Note(
                NoteEvent=Note.Event(
                    note_num=note_num,
                    velocity=0.9,
                    dur_ms=20,
                ),
            ),
            dur_frac=FractionalDuration(num=1, den=16),
        )
    elif word == "x":
        return Event(
            action=Note(
                NoteEvent=Note.Event(
                    note_num=note_num,
                    velocity=0.4,
                    dur_ms=20,
                ),
            ),
            dur_frac=FractionalDuration(num=1, den=16),
        )
    else:
        raise ValueError(f"unsupported notation: {word}")


def _parse(line: str) -> Pattern:
    note_str, events_str = line.strip().split("=")
    note_num = NOTES[note_str.strip()]
    if len(events_str.strip()) == 0:
        return []
    return Pattern(
        name=note_str.strip(),
        events=[
            _parse_event(char, note_num)
            for char in events_str.strip()
            if not char.isspace()
        ],
    )


@click.group
def cli() -> None:
    pass


@cli.command()
def stop() -> None:
    for pattern_name in NOTES:
        pattern = Pattern(name=pattern_name, events=[])
        pattern.stop()


@cli.command()
def start() -> None:
    for line in map(lambda ln: ln.strip(), sys.stdin):
        pattern = _parse(line)
        if len(pattern.events) == 0:
            continue
        pattern.start()


if __name__ == "__main__":
    cli()
