# * Alternation, i.e. the equivalent of "()" in FoxDot and "<>" in tidal.
# * Ties. I think I like tidal's usage of "_".
# * The "@" operator in tidal's mini notation seems very useful ("a@3" -> "a__")
# * Need a way to play multiple notes at once, probably with "()"
#
import itertools
from dataclasses import dataclass
from typing import Any

from lark import Lark, Token, Transformer, Tree

from livecoding.base_types import (
    Duration,
    Event,
    Half,
    Note,
    PluginPattern,
    Rest,
    Sharp,
)
from livecoding.util import random_name


type Octave = int
type PitchClass = int


_DEFAULT_NOTE = 60
_DEFAULT_OCTAVE = 3
_VELOCITY_VALUES = [round((i - 96) / 26, 2) for i in range(97, 123)]
_DEFAULT_VELOCITY = _VELOCITY_VALUES[14]
_PITCH_CLASSES: dict[str, PitchClass] = {
    "C": 0,
    "D": 2,
    "E": 4,
    "F": 5,
    "G": 7,
    "A": 9,
    "B": 11,
}

_LEAF_TYPE = str
_PARSER: Lark | None = None


@dataclass(slots=True)
class Atom:
    value: Rest | Note | float | tuple[int, float]

    def _parse_note_tuple(self, dur: Duration) -> Event:
        assert isinstance(self.value, tuple)
        if len(self.value) == 2:
            if isinstance(self.value[0], int) and isinstance(self.value[1], float):
                return Event(
                    action=Note(
                        Note.Params(
                            note_num=self.value[0], velocity=self.value[1], dur=Half
                        )
                    ),
                    dur=dur,
                )

    def to_event(self, dur: Duration) -> Event:
        if self.value == "Rest":
            return Event(action="Rest", dur=dur)
        elif isinstance(self.value, Note):
            return Event(action=self.value, dur=dur)
        elif isinstance(self.value, int):
            return Event(
                action=Note(
                    Note.Params(
                        note_num=self.value, velocity=_DEFAULT_VELOCITY, dur=Half
                    )
                ),
                dur=dur,
            )
        elif isinstance(self.value, tuple):
            return self._parse_note_tuple(dur)
        assert isinstance(self.value, float), f"{self.value}"
        return Event(
            action=Note(
                Note.Params(dur=Half, note_num=_DEFAULT_NOTE, velocity=self.value)
            ),
            dur=dur,
        )


@dataclass(slots=True)
class AtomRepeated:
    values: list[Atom]


@dataclass(slots=True)
class AtomRepeatedGrouping:
    values: list[Atom]


# Seems like mypy doesn't care about the second generic type for Transformer.
# You can change it from int to something else and mypy doesn't complain.
class PatternTransformer(Transformer[Token, _LEAF_TYPE]):
    def rest(self, value: str) -> Rest:
        return "Rest"

    def atom(self, values: list[Rest | Note | float]) -> Atom:
        assert len(values) == 1
        return Atom(value=values[0])

    def atom_repeated(self, values: tuple[Atom, int]) -> AtomRepeated:
        return AtomRepeated(values=list(itertools.repeat(values[0], int(values[1]))))

    def atom_repeated_grouping(self, values: tuple[Atom, int]) -> AtomRepeatedGrouping:
        return AtomRepeatedGrouping(
            values=list(itertools.repeat(values[0], int(values[1])))
        )

    def velocity(self, value: list[str]) -> float:
        assert len(value) == 1
        return _VELOCITY_VALUES[ord(value[0]) - 97]

    def pitch_class(self, value: list[str]) -> PitchClass:
        return _PITCH_CLASSES[value[0]]

    def octave(self, value: list[str]) -> Octave:
        assert len(value) == 1, f"{value}"
        assert int(value[0]) >= -2 and int(value[0]) <= 7, f"{value}"
        return int(value[0])

    def sharp(self, value: list[str]) -> Sharp:
        return "#"

    def _parse_note1(self, value: list[Any]) -> tuple[int, float]:
        assert len(value) == 1
        return (_get_note_num(value[0], _DEFAULT_OCTAVE), _DEFAULT_VELOCITY)

    def _parse_note2(self, value: list[Any]) -> tuple[int, float]:
        assert len(value) == 2, str(value)
        assert isinstance(value[0], int), str(value)
        # examples: C' or C3 or Ct
        if value[1] == "#":
            return (_get_note_num(value[0] + 1, _DEFAULT_OCTAVE), _DEFAULT_VELOCITY)
        elif isinstance(value[1], int):
            return (_get_note_num(value[0], value[1]), _DEFAULT_VELOCITY)
        assert isinstance(value[1], float)
        return (_get_note_num(value[0], _DEFAULT_OCTAVE), value[1])

    def _parse_note3(self, value: list[Any]) -> tuple[int, float]:
        assert len(value) == 3, str(value)
        assert isinstance(value[0], int), str(value)
        if value[1] == "#":
            if isinstance(value[2], int):
                # example: C'2
                return (_get_note_num(value[0] + 1, value[2]), _DEFAULT_VELOCITY)
            # example: C'w
            assert isinstance(value[2], float), str(value)
            return (_get_note_num(value[0] + 1, _DEFAULT_OCTAVE), value[2])
        # example C2z
        assert isinstance(value[1], int), str(value)
        assert isinstance(value[2], float), str(value)
        return (_get_note_num(value[0], value[1]), value[2])

    def _parse_note4(self, value: list[Any]) -> tuple[int, float]:
        # examples: C'3t
        assert len(value) == 4, str(value)
        assert isinstance(value[0], int), str(value)
        assert value[1] == "#", str(value)
        assert isinstance(value[2], int), str(value)
        assert isinstance(value[3], float), str(value)
        return (_get_note_num(value[0] + 1, value[2]), value[3])

    def note(self, value: list[Any]) -> tuple[int, float]:
        assert len(value) > 0
        if len(value) == 1:
            return self._parse_note1(value)
        elif len(value) == 2:
            return self._parse_note2(value)
        elif len(value) == 3:
            return self._parse_note3(value)
        assert len(value) == 4
        return self._parse_note4(value)


_TRANSFORMER: PatternTransformer | None = None


def _get_note_num(pitch_class: PitchClass, octave: Octave) -> int:
    return pitch_class + ((octave + 2) * 12)


def rhythm_lark_ebnf_v2() -> str:
    return """
    pattern: "[" [ event* ] "]"

    event: pattern
         | atom
         | atom_repeated
         | atom_repeated_grouping

    atom: rest | velocity | note

    atom_repeated: atom ":" INT

    atom_repeated_grouping: atom ";" INT

    !rest: "."

    !velocity: "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z"

    !note: pitch_class sharp? octave? velocity?

    !pitch_class: "A" | "B" | "C" | "D" | "E" | "F" | "G"

    octave: INT

    !sharp: "'"

    %import common.INT
    %import common.WS
    %import common.NUMBER
    %ignore WS
    """


def get_rhythm_parser() -> Lark:
    global _PARSER
    if _PARSER is None:
        _PARSER = Lark(rhythm_lark_ebnf_v2(), start="pattern")
    return _PARSER


def get_rhythm(length_bars: Duration, tree: Tree[_LEAF_TYPE]) -> PluginPattern:
    return PluginPattern(
        events=_get_events(tree, length_bars),
        length_bars=length_bars,
        name=random_name(),
    )


def flatten_repeated(children: list[Any]) -> list[Any]:
    res = []
    for child in children:
        if not isinstance(child, Tree):
            continue
        if len(child.children) != 1:
            continue
        elif isinstance(child.children[0], AtomRepeated):
            # If we are repeating without grouping, then we need to essentially inline
            # the child's children into the child
            res += child.children[0].values
        else:
            res.append(child)  # type: ignore
    # If nothing in the list was a Tree, we should just return the original list.
    return res if len(res) > 0 else children


def _get_events(tree: Tree[_LEAF_TYPE], total_length: Duration) -> list[Event]:
    if len(tree.children) == 0:
        return []
    events: list[Event] = []
    children = flatten_repeated(tree.children)
    each_dur = total_length / len(children)
    for child in children:
        if isinstance(child, Tree):
            events += _get_events(child, each_dur)
            continue
        elif isinstance(child, Atom):
            events.append(child.to_event(each_dur))
        elif isinstance(child, AtomRepeated):
            for atom in child.values:
                events.append(atom.to_event(dur=each_dur / len(child.values)))
        elif isinstance(child, AtomRepeatedGrouping):
            for atom in child.values:
                events.append(atom.to_event(dur=each_dur / len(child.values)))
        elif isinstance(child, str):
            assert child == "Rest"
            events.append(Event(action="Rest", dur=each_dur))
    return events


def get_transformer() -> PatternTransformer:
    global _TRANSFORMER
    if _TRANSFORMER is None:
        _TRANSFORMER = PatternTransformer()
    return _TRANSFORMER


def rhythm_v2(
    definition: str,
    length_bars: Duration = Duration(1, 1),
) -> PluginPattern:
    ast = get_rhythm_parser().parse(definition)
    transformer = get_transformer()
    transformed = transformer.transform(ast)
    return get_rhythm(
        length_bars=length_bars,
        tree=transformed,  # type: ignore
    )
