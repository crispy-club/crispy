import itertools
from collections.abc import Iterator
from dataclasses import dataclass
from typing import Any, NamedTuple

from lark import Lark, Token, Transformer, Tree

from livecoding.base_types import Duration, Event, Note, PluginPattern, Rest
from livecoding.util import random_name


def rhythm_lark_ebnf() -> str:
    return """
    pattern: "[" [ event* ] "]"

    event: pattern
         | rest
         | rest_repeated
         | velocity
         | velocity_repeated

    rest: "~"

    rest_repeated: rest "*" INT

    velocity: NUMBER

    velocity_repeated: velocity "*" INT

    %import common.INT
    %import common.WS
    %import common.NUMBER
    %ignore WS
    """


class Hit(NamedTuple):
    velocity: float
    dur: Duration


@dataclass(slots=True)
class RestFor:
    dur: Duration


def _to_event(hitr: Hit | RestFor, note_numbers: Iterator[int]) -> Event:
    if isinstance(hitr, RestFor):
        return Event(action="Rest", dur=hitr.dur)
    return Event(
        action=Note(
            Note.Params(
                note_num=next(note_numbers), velocity=hitr.velocity, dur=Duration(1, 2)
            )
        ),
        dur=hitr.dur,
    )


@dataclass(slots=True)
class Rhythm:
    hits: list[Hit | RestFor]
    length_bars: Duration

    def _get_events(self, note_numbers: Iterator[int]) -> list[Event]:
        note_nums = itertools.cycle(note_numbers)
        return [_to_event(hitr, note_nums) for hitr in self.hits]

    def __ror__(self, note_numbers: Iterator[int]) -> PluginPattern:
        return PluginPattern(
            events=self._get_events(note_numbers),
            length_bars=self.length_bars,
            name=random_name(),
        )


_PARSER: Lark | None = None


def get_rhythm_parser() -> Lark:
    global _PARSER
    if _PARSER is None:
        _PARSER = Lark(rhythm_lark_ebnf(), start="pattern")
    return _PARSER


_LEAF_TYPE = float | tuple[float, int]


def get_rhythm(length_bars: Duration, tree: Tree[_LEAF_TYPE]) -> Rhythm:
    return Rhythm(
        hits=_get_hits(tree, length_bars),
        length_bars=length_bars,
    )


def flatten_repeated(children: list[Any]) -> list[Any]:
    res = []
    for child in children:
        if not isinstance(child, Tree):
            continue
        if len(child.children) != 1:
            continue
        if isinstance(child.children[0], RestRepeated):
            res += ["Rest"] * child.children[0].repeats
        elif isinstance(child.children[0], VelocityRepeated):
            res += child.children[0].value  # type: ignore
        else:
            res.append(child)  # type: ignore
    # If nothing in the list was a Tree, we should just return the original list.
    return res if len(res) > 0 else children


def _get_hits(tree: Tree[_LEAF_TYPE], total_length: Duration) -> list[Hit | RestFor]:
    if len(tree.children) == 0:
        return []
    hits: list[Hit | RestFor] = []
    children = flatten_repeated(tree.children)
    each_dur = total_length / len(children)
    for child in children:
        if isinstance(child, Tree):
            hits += _get_hits(child, each_dur)
            continue
        elif isinstance(child, float):
            assert 0.0 <= child and child <= 1.0
            hits.append(Hit(velocity=child, dur=each_dur))
        elif isinstance(child, VelocityRepeated):
            for velocity in child.value:
                hits.append(Hit(dur=each_dur / len(child.value), velocity=velocity))
        elif isinstance(child, RestRepeated):
            for _ in range(child.repeats):
                hits.append(RestFor(dur=each_dur / child.repeats))
        elif isinstance(child, str):
            assert child == "Rest"
            hits.append(RestFor(dur=each_dur))
    return hits


@dataclass(slots=True)
class VelocityRepeated:
    value: list[float]


@dataclass(slots=True)
class RestRepeated:
    repeats: int


# Seems like mypy doesn't care about the second generic type for Transformer.
# You can change it from int to something else and mypy doesn't complain.
class PatternTransformer(Transformer[Token, _LEAF_TYPE]):
    def rest(self, value: str) -> Rest:
        return "Rest"

    def rest_repeated(self, value: list[str]) -> RestRepeated:
        assert len(value) == 2
        return RestRepeated(repeats=int(value[1]))

    def velocity(self, value: list[str]) -> float:
        assert len(value) == 1
        fv = float(value[0])
        assert fv >= 0.0 and fv <= 1.0
        return fv

    def velocity_repeated(self, value: tuple[int, Token]) -> VelocityRepeated:
        return VelocityRepeated(
            list(itertools.repeat(float(value[0]), int(value[1].value)))
        )


_TRANSFORMER: PatternTransformer | None = None


def get_transformer() -> PatternTransformer:
    global _TRANSFORMER
    if _TRANSFORMER is None:
        _TRANSFORMER = PatternTransformer()
    return _TRANSFORMER


def rhythm(
    definition: str,
    length_bars: Duration = Duration(1, 1),
) -> Rhythm:
    """
    note_pattern parses the melody DSL, which is very similar in spirit to the
    tidal cycles "mini notation"
    """
    ast = get_rhythm_parser().parse(definition)
    print(f"ast -> {ast}")
    transformer = get_transformer()
    transformed = transformer.transform(ast)
    print(f"transformed -> {transformed}")
    return get_rhythm(
        length_bars=length_bars,
        tree=transformed,  # type: ignore
    )
