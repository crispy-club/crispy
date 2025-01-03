import itertools

from attrs import define
from lark import Lark, Token, Transformer, Tree
from wonderwords import RandomWord, Defaults

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

    rest: "~"

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


def _random_name() -> str:
    adj, noun = RandomWord(adj=Defaults.ADJECTIVES), RandomWord(adj=Defaults.NOUNS)
    return f"{adj.word()}-{noun.word()}"


def get_note_pattern(
    length_bars: Duration, tree: Tree[_LEAF_TYPE], default_velocity: float
) -> NotePattern:
    return NotePattern(
        name=_random_name(),
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
                    action=Note(Note.Params(child, default_velocity, Duration(1, 2))),
                    dur=each_dur,
                )
            )
        elif isinstance(child, tuple):
            assert len(child) == 2
            events.append(
                Event(
                    action=Note(Note.Params(child[0], child[1], Duration(1, 2))),
                    dur=each_dur,
                )
            )
        elif isinstance(child, NoteRepeated):
            for note_num in child.value:
                events.append(
                    Event(
                        action=Note(
                            Note.Params(note_num, default_velocity, Duration(1, 2))
                        ),
                        dur=each_dur / len(child.value),
                    )
                )
        elif isinstance(child, str):
            assert child == "Rest"
            events.append(
                Event(
                    action="Rest",
                    dur=each_dur,
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


def notes(
    definition: str,
    length_bars: Duration = Duration(1, 1),
    default_velocity: float = 0.8,
) -> NotePattern:
    """
    note_pattern parses the melody DSL, which is very similar in spirit to the
    tidal cycles "mini notation"
    """
    ast = get_pattern_parser().parse(definition)
    transformer = get_transformer()
    return get_note_pattern(
        length_bars=length_bars,
        tree=transformer.transform(ast),  # type: ignore
        default_velocity=default_velocity,
    )


@define
class Perc:
    @classmethod
    def parse(cls, line: str) -> NotePattern:
        note_str, events_str = line.strip().split("=")
        note_num = NoteNumbers[note_str.strip()]
        if len(events_str.strip()) == 0:
            return NotePattern(name="", length_bars=Duration(0, 1), events=[])
        events = [
            cls.parse_event(char, note_num)
            for char in events_str.strip()
            if not char.isspace()
        ]
        return NotePattern(
            name=note_str.strip(),
            events=events,
            length_bars=Duration(len(events), 16),
        )

    @classmethod
    def parse_event(cls, word: str, note_num: int) -> Event:
        if word == ".":
            return Event(action="Rest", dur=Duration(num=1, den=16))
        elif word == "X":
            return Event(
                action=Note(
                    Note.Params(
                        note_num=note_num,
                        velocity=0.9,
                        dur=Duration(1, 2),
                    ),
                ),
                dur=Duration(num=1, den=16),
            )
        elif word == "x":
            return Event(
                action=Note(
                    Note.Params(
                        note_num=note_num,
                        velocity=0.4,
                        dur=Duration(1, 2),
                    ),
                ),
                dur=Duration(num=1, den=16),
            )
        elif word == "_":
            return Event(
                action="Tie",
                dur=Duration(num=1, den=16),
            )
        else:
            raise ValueError(f"unsupported notation: {word}")

    @classmethod
    def usage(cls) -> str:
        return "NOTE = [Xx_.]"


def perc(
    definition: str,
) -> list[NotePattern]:
    """
    perc_pattern parses a DSL that is geared towards linear sequencing
    of individual notes.
    """
    try:
        return [
            Perc.parse(line.strip())
            for line in definition.split("\n")
            if len(line.strip()) > 0
        ]
    except ValueError as ex:
        print(f"could not parse pattern: {ex}")
        print(f"expected line format: {Perc.usage()}")
        raise ex
