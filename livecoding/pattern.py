from collections import deque

from attrs import define

from livecoding.base_types import Duration, Event, Note, NotePattern


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
        return Event(
            dur_frac=event.dur_frac, action=event.action.transpose(self.amount)
        )


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
    running_total = Duration(0, 1)
    for idx, event in enumerate(events):
        running_total += event.dur_frac
        if running_total == length_bars:
            if idx < len(events):
                return events[: idx + 1]
            return events
        elif running_total > length_bars:
            remainder = running_total - length_bars
            return events[:idx] + [Event(action=events[idx].action, dur_frac=remainder)]
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
                    dur_frac=ev.dur_frac * self.scalar,
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
