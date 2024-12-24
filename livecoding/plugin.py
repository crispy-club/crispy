import jsonplus
import requests

from livecoding.base_types import Duration, Event, Pattern


jsonplus.prefer_compat()


def _gen_random_name() -> str:
    return "foo"


def _compile(pattern_definition: str) -> list[Event]:
    return []


def pat(
    name: str,
    pattern_definition: str,
    length_bars: Duration = Duration(1, 1),
    default_velocity: float = 0.8,
    default_duration: Duration = Duration(1, 16),
) -> Pattern:
    if name is None:
        name = _gen_random_name()
    return Pattern(
        length_bars=length_bars,
        name=name,
        events=_compile(pattern_definition),
    )


def stop(pattern_name: str) -> None:
    resp = requests.post(f"http://127.0.0.1:3000/stop/{pattern_name}")
    resp.raise_for_status()


def play(pattern: Pattern) -> None:
    data = jsonplus.dumps({"events": pattern.events})
    resp = requests.post(
        f"http://127.0.0.1:3000/start/{pattern.name}",
        headers={"Content-Type": "application/json"},
        data=data,
    )
    resp.raise_for_status()
