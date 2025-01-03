import json

import requests
from attrs import asdict, define

from livecoding.base_types import NotePattern


@define
class _Channel:
    """
    ch1.play(notes("[c0 d0 e0 f0]") | rev | resize(Bar * 4))
    """

    n: int


def stop(pattern_name: str) -> None:
    resp = requests.post(f"http://127.0.0.1:3000/stop/{pattern_name}")
    resp.raise_for_status()


def play(*patterns: NotePattern) -> None:
    for pattern in patterns:
        data = json.dumps({"events": [asdict(ev) for ev in pattern.events]})
        resp = requests.post(
            f"http://127.0.0.1:3000/start/{pattern.name}",
            headers={"Content-Type": "application/json"},
            data=data,
        )
        resp.raise_for_status()
