import jsonplus
import requests

from livecoding.base_types import Duration, Pattern
from livecoding.grammar import get_pattern_parser
from livecoding.pattern import _get_pattern, _get_transformer


jsonplus.prefer_compat()


def note_pattern(
    name: str,
    pattern_definition: str,
    length_bars: Duration = Duration(1, 1),
    default_velocity: float = 0.8,
) -> Pattern:
    ast = get_pattern_parser().parse(pattern_definition)
    transformer = _get_transformer()
    return _get_pattern(
        name=name,
        length_bars=length_bars,
        tree=transformer.transform(ast),  # type: ignore
        default_velocity=default_velocity,
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
