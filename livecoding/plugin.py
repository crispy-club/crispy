import json

import requests
from attrs import asdict

from livecoding.base_types import Duration, NotePattern
from livecoding.grammar import get_pattern_parser, get_note_pattern, get_transformer


def note_pattern(
    name: str,
    definition: str,
    length_bars: Duration = Duration(1, 1),
    default_velocity: float = 0.8,
) -> NotePattern:
    ast = get_pattern_parser().parse(definition)
    transformer = get_transformer()
    return get_note_pattern(
        name=name,
        length_bars=length_bars,
        tree=transformer.transform(ast),  # type: ignore
        default_velocity=default_velocity,
    )


def stop(pattern_name: str) -> None:
    resp = requests.post(f"http://127.0.0.1:3000/stop/{pattern_name}")
    resp.raise_for_status()


def play(pattern: NotePattern) -> None:
    data = json.dumps({"events": [asdict(ev) for ev in pattern.events]})
    resp = requests.post(
        f"http://127.0.0.1:3000/start/{pattern.name}",
        headers={"Content-Type": "application/json"},
        data=data,
    )
    resp.raise_for_status()
