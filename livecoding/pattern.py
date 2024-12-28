from collections import deque

from attrs import define
from lark import Token, Transformer, Tree

from livecoding.base_types import Duration, Event, Note, NotePattern, Rest
from livecoding.notes import NoteNumbers


_LEAF_TYPE = int | tuple[int, int]


def _get_pattern(
    name: str, length_bars: Duration, tree: Tree[_LEAF_TYPE], default_velocity: float
) -> NotePattern:
    return NotePattern(
        name=name,
        length_bars=length_bars,
        events=_get_events(tree, default_velocity, length_bars),
    )


def _get_events(
    tree: Tree[_LEAF_TYPE], default_velocity: float, total_length: Duration
) -> list[Event]:
    if len(tree.children) == 0:
        return []
    events: list[Event] = []
    each_dur = total_length / len(tree.children)
    for child in tree.children:
        if isinstance(child, Tree):
            events += _get_events(child, default_velocity, each_dur)
            continue
        if isinstance(child, int):
            events.append(
                Event(
                    action=Note(Note.Params(child, default_velocity, 20)),
                    dur_frac=each_dur,
                )
            )  # Need to be smarter about dur_ms
        elif isinstance(child, tuple):
            assert len(child) == 2
            events.append(
                Event(
                    action=Note(Note.Params(child[0], child[1], 20)),
                    dur_frac=each_dur,
                )
            )
        elif isinstance(child, str):
            assert child == "Rest"
            events.append(
                Event(
                    action="Rest",
                    dur_frac=each_dur,
                )
            )
    return events


# Seems like mypy doesn't care about the second generic type for Transformer.
# You can change it from int to something else and mypy doesn't complain.
class _PatternTransformer(Transformer[Token, _LEAF_TYPE]):
    def rest(self, value: str) -> Rest:
        return "Rest"

    def note(self, value: list[str]) -> int:
        assert len(value) == 1
        # No risk of KeyError here since the grammar enforces valid note numbers
        return NoteNumbers[value[0]]

    def velocity(self, value: list[str]) -> float:
        assert len(value) == 1
        return float(value[0])

    def pair(self, value: list[int]) -> tuple[int, int]:
        assert len(value) == 2
        return (value[0], value[1])


_TRANSFORMER: _PatternTransformer | None = None


def _get_transformer() -> _PatternTransformer:
    global _TRANSFORMER
    if _TRANSFORMER is None:
        _TRANSFORMER = _PatternTransformer()
    return _TRANSFORMER


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
        if event.action == Rest:
            return event
        elif isinstance(event.action, Note):
            return Event(
                dur_frac=event.dur_frac, action=event.action.transpose(self.amount)
            )
        else:
            raise ValueError(f"unknown event type: {event}")


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
        if running_total >= length_bars:
            return events[: idx - 1] + [
                Event(action=event.action, dur_frac=running_total - length_bars)
            ]
    return events


@define
class lclip:
    length_bars: Duration

    def __call__(self, pattern: NotePattern) -> NotePattern:
        return NotePattern(
            name=pattern.name,
            length_bars=pattern.length_bars,
            events=list(
                reversed(_right_clip(self.length_bars, list(reversed(pattern.events))))
            ),
        )


@define
class rclip:
    length_bars: Duration

    def __call__(self, pattern: NotePattern) -> NotePattern:
        return NotePattern(
            name=pattern.name,
            length_bars=pattern.length_bars,
            events=_right_clip(self.length_bars, pattern.events),
        )


@define
class ladd:
    ev: Event

    def __call__(self, pattern: NotePattern) -> NotePattern:
        return NotePattern(
            name=pattern.name,
            length_bars=pattern.length_bars + self.ev.dur_frac,
            events=[self.ev] + pattern.events,
        )


@define
class radd:
    ev: Event

    def __call__(self, pattern: NotePattern) -> NotePattern:
        return NotePattern(
            name=pattern.name,
            length_bars=pattern.length_bars + self.ev.dur_frac,
            events=pattern.events + [self.ev],
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
