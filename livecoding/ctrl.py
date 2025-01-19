import itertools
from typing import Iterable

from attrs import define

from livecoding.base_types import Bar, Ctrl, Duration, Event, PluginPattern, Sixteenth
from livecoding.pattern import name
from livecoding.plugin import play
from livecoding.util import random_name


@define
class CC:
    channel: int
    number: int

    def __lshift__(self, pattern: PluginPattern) -> None:
        play(pattern | name(f"cc{self.number}"), channel=self.channel)


@define
class CCEvent:
    cc: int
    value: float


def _ctrl_events(values: Iterable[CCEvent], rhythm: Iterable[Duration]) -> list[Event]:
    durations = itertools.cycle(rhythm)
    return [
        Event(action=Ctrl(Ctrl.Params(cc=cev.cc, value=cev.value)), dur=next(durations))
        for cev in values
    ]


def ccp(
    values: Iterable[CCEvent],
    channel: int = 1,
    rhythm: Iterable[Duration] | None = None,
    length_bars: Duration = Bar,
) -> PluginPattern:
    if rhythm is None:
        rhythm = [Sixteenth]
    return PluginPattern(
        name=random_name(), length_bars=length_bars, events=_ctrl_events(values, rhythm)
    )
