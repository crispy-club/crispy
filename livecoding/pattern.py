import functools
import itertools
import operator
from collections import deque

from attrs import define

from livecoding.base_types import (
    Duration,
    Event,
    Half,
    Note,
    NotePattern,
    NotePatternFilter,
    Sixteenth,
    Zero,
)
from livecoding.notes import NoteNumbers


@define
class Perc:
    previous_event: Event | None = None

    def parse(self, definition: str) -> list[NotePattern]:
        return [
            self.parse_line(line.strip())
            for line in definition.split("\n")
            if len(line.strip()) > 0
        ]

    def parse_line(self, line: str) -> NotePattern:
        self.previous_event = None
        note_str, events_str = line.strip().split("=")
        note_num = NoteNumbers[note_str.strip()]
        if len(events_str.strip()) == 0:
            return NotePattern(name=note_str.strip(), length_bars=Zero, events=[])
        events = [
            event
            for event in [
                self.parse_event(char, note_num)
                for char in events_str.strip()
                if not char.isspace()
            ]
            if event is not None
        ]
        return NotePattern(
            name=note_str.strip(),
            events=events,
            length_bars=functools.reduce(operator.add, [ev.dur for ev in events]),
        )

    def parse_event(self, word: str, note_num: int) -> Event | None:
        if word == ".":
            event = Event(action="Rest", dur=Sixteenth)
        elif word == "X":
            event = Event(
                action=Note(
                    Note.Params(
                        note_num=note_num,
                        velocity=0.9,
                        dur=Half,
                    ),
                ),
                dur=Sixteenth,
            )
        elif word == "x":
            event = Event(
                action=Note(
                    Note.Params(
                        note_num=note_num,
                        velocity=0.4,
                        dur=Half,
                    ),
                ),
                dur=Sixteenth,
            )
        elif word == "+":
            assert self.previous_event is not None, "no event to tie"
            # lengthen the previous event
            self.previous_event.dur += Sixteenth
            return None
        else:
            raise ValueError(
                f"unsupported notation: {word} (line format is {self.usage()})"
            )
        self.previous_event = event
        return event

    def usage(cls) -> str:
        return "NOTE = [Xx_.]"


def perc(
    definition: str,
) -> list[NotePattern]:
    """
    perc_pattern parses a DSL that is geared towards linear sequencing
    of individual notes.
    """
    _perc = Perc()

    try:
        return _perc.parse(definition)
    except ValueError as ex:
        print(f"could not parse pattern: {ex}")
        print(f"expected line format: {_perc.usage()}")
        raise ex


@define
class _rev:
    def __call__(self, pattern: NotePattern) -> NotePattern:
        return NotePattern(
            events=list(reversed(pattern.events)),
            length_bars=pattern.length_bars,
            name=pattern.name,
        )


rev = _rev()


@define
class tran:
    amount: int

    def __call__(self, pattern: NotePattern) -> NotePattern:
        return NotePattern(
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


@define
class rot:
    n: int

    def __call__(self, pattern: NotePattern) -> NotePattern:
        events = deque(pattern.events)
        events.rotate(self.n)
        return NotePattern(
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


@define
class lclip:
    length_bars: Duration

    def __call__(self, pattern: NotePattern) -> NotePattern:
        assert self.length_bars < pattern.length_bars
        new_length = pattern.length_bars - self.length_bars
        return NotePattern(
            name=pattern.name,
            length_bars=new_length,
            events=list(
                reversed(_right_clip(new_length, list(reversed(pattern.events))))
            ),
        )


@define
class rclip:
    length_bars: Duration

    def __call__(self, pattern: NotePattern) -> NotePattern:
        assert self.length_bars < pattern.length_bars
        new_length = pattern.length_bars - self.length_bars
        return NotePattern(
            name=pattern.name,
            length_bars=new_length,
            events=_right_clip(new_length, pattern.events),
        )


@define
class ladd:
    pattern: NotePattern

    def __call__(self, pattern: NotePattern) -> NotePattern:
        return NotePattern(
            name=pattern.name,
            length_bars=pattern.length_bars + self.pattern.length_bars,
            events=self.pattern.events + pattern.events,
        )


@define
class radd:
    pattern: NotePattern

    def __call__(self, pattern: NotePattern) -> NotePattern:
        return NotePattern(
            name=pattern.name,
            length_bars=pattern.length_bars + self.pattern.length_bars,
            events=pattern.events + self.pattern.events,
        )


@define
class resize:
    scalar: Duration

    def __call__(self, pattern: NotePattern) -> NotePattern:
        return NotePattern(
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


@define
class name:
    new_name: str

    def __call__(self, pattern: NotePattern) -> NotePattern:
        return NotePattern(
            name=self.new_name,
            length_bars=pattern.length_bars,
            events=pattern.events,
        )


@define
class revery:
    n: int
    filt: NotePatternFilter

    def __call__(self, pattern: NotePattern) -> NotePattern:
        assert self.n > 1
        return functools.reduce(
            operator.add, itertools.repeat(pattern, self.n - 1)
        ) + self.filt(pattern)


@define
class levery:
    n: int
    filt: NotePatternFilter

    def __call__(self, pattern: NotePattern) -> NotePattern:
        assert self.n > 1
        return self.filt(pattern) + functools.reduce(
            operator.add, itertools.repeat(pattern, self.n - 1)
        )
