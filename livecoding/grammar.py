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


def create_pattern_parser() -> Lark:
    return Lark(lark_ebnf(), start="pattern")
