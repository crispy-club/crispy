import copy
import functools
import itertools
import operator
from collections import deque
from dataclasses import dataclass
from typing import Callable

from crispy.base_types import (
    Duration,
    Event,
    Note,
    PluginPattern,
    PluginPatternFilter,
    Zero,
)


@dataclass(slots=True)
class _rev:
    def __call__(self, pattern: PluginPattern) -> PluginPattern:
        return PluginPattern(
            events=list(reversed(pattern.events)),
            length_bars=pattern.length_bars,
            name=pattern.name,
        )


rev = _rev()


@dataclass(slots=True)
class tran:
    amount: int

    def __call__(self, pattern: PluginPattern) -> PluginPattern:
        return PluginPattern(
            events=list(map(lambda ev: self._transpose(ev), pattern.events)),
            length_bars=pattern.length_bars,
            name=pattern.name,
        )

    def _transpose(self, event: Event) -> Event:
        assert isinstance(self.amount, int)
        if event.action == "Rest":
            return event
        assert isinstance(event.action, Note)
        return Event(dur=event.dur, action=event.action.transpose(self.amount))


@dataclass(slots=True)
class rot:
    n: int

    def __call__(self, pattern: PluginPattern) -> PluginPattern:
        events = deque(pattern.events)
        events.rotate(self.n)
        return PluginPattern(
            name=pattern.name,
            length_bars=pattern.length_bars,
            events=list(events),
        )


def _right_clip(length_bars: Duration, events: list[Event]) -> list[Event]:
    running_total = Zero
    for idx, event in enumerate(events):
        running_total += event.dur
        if running_total == length_bars:
            return events[: idx + 1]
        elif running_total > length_bars:
            remainder = running_total - length_bars
            return events[:idx] + [Event(action=events[idx].action, dur=remainder)]
    return events


@dataclass(slots=True)
class lclip:
    length_bars: Duration

    def __call__(self, pattern: PluginPattern) -> PluginPattern:
        assert self.length_bars < pattern.length_bars
        new_length = pattern.length_bars - self.length_bars
        return PluginPattern(
            name=pattern.name,
            length_bars=new_length,
            events=list(
                reversed(_right_clip(new_length, list(reversed(pattern.events))))
            ),
        )


@dataclass(slots=True)
class rclip:
    length_bars: Duration

    def __call__(self, pattern: PluginPattern) -> PluginPattern:
        assert self.length_bars < pattern.length_bars
        new_length = pattern.length_bars - self.length_bars
        return PluginPattern(
            name=pattern.name,
            length_bars=new_length,
            events=_right_clip(new_length, pattern.events),
        )


@dataclass(slots=True)
class ladd:
    pattern: PluginPattern

    def __call__(self, pattern: PluginPattern) -> PluginPattern:
        return PluginPattern(
            name=pattern.name,
            length_bars=pattern.length_bars + self.pattern.length_bars,
            events=self.pattern.events + pattern.events,
        )


@dataclass(slots=True)
class radd:
    pattern: PluginPattern

    def __call__(self, pattern: PluginPattern) -> PluginPattern:
        return PluginPattern(
            name=pattern.name,
            length_bars=pattern.length_bars + self.pattern.length_bars,
            events=pattern.events + self.pattern.events,
        )


@dataclass(slots=True)
class resize:
    scalar: Duration

    def __call__(self, pattern: PluginPattern) -> PluginPattern:
        return PluginPattern(
            name=pattern.name,
            length_bars=self.scalar * pattern.length_bars,
            events=[
                Event(
                    action=ev.action,
                    dur=ev.dur * self.scalar,
                )
                for ev in pattern.events
            ],
        )


@dataclass(slots=True)
class name:
    new_name: str

    def __call__(self, pattern: PluginPattern) -> PluginPattern:
        return PluginPattern(
            name=self.new_name,
            length_bars=pattern.length_bars,
            events=pattern.events,
        )


@dataclass(slots=True)
class revery:
    n: int
    filt: PluginPatternFilter

    def __call__(self, pattern: PluginPattern) -> PluginPattern:
        assert self.n > 1
        return functools.reduce(
            operator.add, itertools.repeat(pattern, self.n - 1)
        ) + self.filt(pattern)


@dataclass(slots=True)
class levery:
    n: int
    filt: PluginPatternFilter

    def __call__(self, pattern: PluginPattern) -> PluginPattern:
        assert self.n > 1
        return self.filt(pattern) + functools.reduce(
            operator.add, itertools.repeat(pattern, self.n - 1)
        )


EventFilter = Callable[[Event], Event]
NoteFilter = Callable[[Note], Note]


@dataclass(slots=True)
class each:
    f: EventFilter

    def __call__(self, pattern: PluginPattern) -> PluginPattern:
        new_events = list(map(self.f, pattern.events))
        new_length = sum(map(lambda ev: ev.dur, new_events), start=Zero)
        return copy.replace(
            copy.replace(pattern, events=list(map(self.f, pattern.events))),
            length_bars=new_length,
        )


@dataclass(slots=True)
class each_note:
    f: NoteFilter

    def _eventfilter(self) -> EventFilter:
        def filt(event: Event) -> Event:
            if event.is_ctrl() or event.is_rest():
                return event
            assert isinstance(event.action, Note)
            return copy.replace(event, action=self.f(event.action))

        return filt

    def __call__(self, pattern: PluginPattern) -> PluginPattern:
        return copy.replace(
            pattern, events=list(map(self._eventfilter(), pattern.events))
        )
