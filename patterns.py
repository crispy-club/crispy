import sys

from typing import NamedTuple

import jsonplus
import requests


jsonplus.prefer_compat()


NOTES = {
    "c1": 36,
    "d1": 38,
    "e1": 40,
    "f1": 41,
}


class Note(NamedTuple):
    note_num: int
    velocity: float
    dur_ms: int


class Event(NamedTuple):
    note: Note
    dur_beats: float


class Pattern(NamedTuple):
    events: list[Event]


def _post(url: str, pattern: Pattern) -> None:
    data = jsonplus.dumps(pattern)
    resp = requests.post(url, headers={"Content-Type": "application/json"}, data=data)
    resp.raise_for_status()


def _parse_event(word: str) -> Event | None:
    parts = word.split(":")
    assert len(parts) > 0
    note_num = NOTES[parts[0]]
    dur_beats = float(parts[1]) if len(parts) > 1 else 1.0
    return Event(
        note=Note(
            note_num=note_num,
            velocity=0.8,
            dur_ms=20,
        ),
        dur_beats=dur_beats,
    )


def _parse(line: str) -> list[Event]:
    if len(line.strip()) == 0:
        return []
    return [_parse_event(word) for word in line.strip().split()]


def main() -> None:
    events: list[Event] = []
    for line in map(lambda ln: ln.strip(), sys.stdin):
        ln_events = _parse(line)
        if len(ln_events) == 0:
            continue
        events += ln_events
    _post("http://127.0.0.1:3000/start", Pattern(events=events))


if __name__ == "__main__":
    main()
