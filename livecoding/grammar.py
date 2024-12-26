import itertools

from lark import Lark

from .notes import NoteNumbers


def lark_ebnf() -> str:
    notes = """
          | """.join(
        " | ".join((f'"{n}"' for n in notes))
        for notes in itertools.batched(NoteNumbers, n=12)
    )

    velocity_values = """
            | """.join(
        " | ".join((f'"{str(i)}"' for i in batch))
        for batch in itertools.batched(iter(range(128)), n=16)
    )

    return f"""
    pattern: "[" [ event* ] "]"

    event: pattern
         | rest
         | note
         | pair

    rest: "."

    !note: {notes}

    !velocity: {velocity_values}

    duration: INT "/" INT

    pair: note "," velocity

    %import common.INT
    %import common.WS
    %ignore WS
    """


_PARSER: Lark | None = None


def get_pattern_parser() -> Lark:
    global _PARSER
    if _PARSER is None:
        _PARSER = Lark(lark_ebnf(), start="pattern")
    return _PARSER
