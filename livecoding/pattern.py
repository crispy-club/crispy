from lark import Token, Transformer

from livecoding.base_types import Duration, Rest
from livecoding.notes import NoteNumbers


# Seems like mypy doesn't care about the second generic type for Transformer.
# You can change it from int to something else and mypy doesn't complain.
class PatternTransformer(Transformer[Token, int]):
    def rest(self) -> Rest:
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
