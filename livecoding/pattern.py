from lark import Token, Transformer, Tree

from livecoding.base_types import Duration, Event, Note, Pattern, Rest
from livecoding.notes import NoteNumbers


_LEAF_TYPE = int | tuple[int, int]


def _get_pattern(
    name: str, length_bars: Duration, tree: Tree[_LEAF_TYPE], default_velocity: float
) -> Pattern:
    return Pattern(
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
            assert len(events) == 2
            events.append(
                Event(
                    action=Note(Note.Params(child[0], child[1], 20)),
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
        note_num = NoteNumbers.get(value[0])
        if note_num is None:
            raise ValueError(f"unknown note value {value[0]}")
        return note_num

    def velocity(self, value: list[str]) -> int:
        assert len(value) == 1
        return int(value[0])

    def duration(self, value: list[str]) -> Duration:
        assert len(value) == 2
        return Duration(int(value[0]), int(value[1]))

    def pair(self, value: list[int]) -> tuple[int, int]:
        assert len(value) == 2
        return (value[0], value[1])

    def triple(self, value: tuple[int, int, Duration]) -> tuple[int, int, Duration]:
        assert len(value) == 3
        return (
            value[0],
            value[1],
            value[2],
        )


_TRANSFORMER: _PatternTransformer | None = None


def _get_transformer() -> _PatternTransformer:
    global _TRANSFORMER
    if _TRANSFORMER is None:
        _TRANSFORMER = _PatternTransformer()
    return _TRANSFORMER
