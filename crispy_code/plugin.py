import json
from dataclasses import asdict, dataclass
from typing import Any

import requests

from crispy.base_types import PluginPattern, Zero
from crispy.filters import name


@dataclass(slots=True)
class Channel:
    """
    ch1 << notes("[c0 d0 e0 f0]") | rev | resize(Bar * 4))
    """

    n: int

    def __lshift__(self, pattern: PluginPattern) -> None:
        play(pattern | name(f"ch{self.n}"), channel=self.n)

    def stop(self) -> None:
        stop(f"ch{self.n}")


ch1 = Channel(1)
ch2 = Channel(2)
ch3 = Channel(3)
ch4 = Channel(4)
ch5 = Channel(5)
ch6 = Channel(6)
ch7 = Channel(7)
ch8 = Channel(8)
ch9 = Channel(9)
ch10 = Channel(10)
ch11 = Channel(11)
ch12 = Channel(12)
ch13 = Channel(13)
ch14 = Channel(14)
ch15 = Channel(15)
ch16 = Channel(16)


def stop(pattern_name: str) -> None:
    resp = requests.post(f"http://127.0.0.1:3000/stop/{pattern_name}")
    resp.raise_for_status()


def play(*patterns: PluginPattern, channel: int | None = None) -> None:
    for pattern in patterns:
        _play(pattern, channel)


def _play(pattern: PluginPattern, channel: int | None = None) -> None:
    assert (
        sum(map(lambda ev: ev.dur, pattern.events), start=Zero) == pattern.length_bars
    )
    body: dict[str, Any] = {"events": [asdict(ev) for ev in pattern.events]}
    if channel is not None:
        assert channel > 0 and channel <= 16
        body["channel"] = channel
    else:
        body["channel"] = 1
    data = json.dumps(body)
    resp = requests.post(
        f"http://127.0.0.1:3000/start/{pattern.name}",
        headers={"Content-Type": "application/json"},
        data=data,
    )
    resp.raise_for_status()
    print(pattern.name)
