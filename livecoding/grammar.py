import itertools

from attrs import define
from lark import Lark, Token, Transformer, Tree

from livecoding.base_types import Duration, Event, Note, NotePattern, Rest
from livecoding.notes import NoteNumbers


def lark_ebnf() -> str:
    notes = """
          | """.join(
        " | ".join((f'"{n}"' for n in notes))
        for notes in itertools.batched(NoteNumbers, n=12)
    )

    return f"""
    pattern: "[" [ event* ] "]"

    event: pattern
         | rest
         | note
         | note_repeated
         | pair
         | pair_repeated

    rest: "."

    !note: {notes}

    note_repeated: note "*" INT

    velocity: NUMBER

    pair: note "," velocity

    pair_repeated: "(" note "," velocity ")" "*" INT

    %import common.INT
    %import common.WS
    %import common.NUMBER
    %ignore WS
    """


_PARSER: Lark | None = None


def get_pattern_parser() -> Lark:
    global _PARSER
    if _PARSER is None:
        _PARSER = Lark(lark_ebnf(), start="pattern")
    return _PARSER


_LEAF_TYPE = int | tuple[int, int]


def get_note_pattern(
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
        elif isinstance(child, NoteRepeated):
            for note_num in child.value:
                events.append(
                    Event(
                        action=Note(Note.Params(note_num, default_velocity, 20)),
                        dur_frac=each_dur / len(child.value),
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


@define
class NoteRepeated:
    value: list[int]


# Seems like mypy doesn't care about the second generic type for Transformer.
# You can change it from int to something else and mypy doesn't complain.
class PatternTransformer(Transformer[Token, _LEAF_TYPE]):
    def rest(self, value: str) -> Rest:
        return "Rest"

    def note(self, value: list[str]) -> int:
        assert len(value) == 1
        # No risk of KeyError here since the grammar enforces valid note numbers
        return NoteNumbers[value[0]]

    def note_repeated(self, value: tuple[int, Token]) -> NoteRepeated:
        print(f"value {value}")
        print(f"type(value) {type(value)}")
        return NoteRepeated(list(itertools.repeat(int(value[0]), int(value[1].value))))

    def velocity(self, value: list[str]) -> float:
        assert len(value) == 1
        fv = float(value[0])
        assert fv >= 0 and fv <= 1
        return fv

    def pair(self, value: list[int]) -> tuple[int, int]:
        assert len(value) == 2
        return (value[0], value[1])


_TRANSFORMER: PatternTransformer | None = None


def get_transformer() -> PatternTransformer:
    global _TRANSFORMER
    if _TRANSFORMER is None:
        _TRANSFORMER = PatternTransformer()
    return _TRANSFORMER
