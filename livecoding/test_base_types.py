# import functools

from livecoding.base_types import Bar, Duration, Note
from livecoding.pattern import resize
from livecoding.plugin import note_pattern


def test_duration_division() -> None:
    assert Bar / 4 == Duration(1, 4)
    half_note = Bar / 2
    quarter_note = Bar / 4
    assert half_note / quarter_note == Bar * 2
    assert 1 / quarter_note == Bar * 4


def test_duration_subtraction() -> None:
    assert Duration(1, 4) - Duration(1, 8) == Duration(1, 8)


def test_note_eq() -> None:
    n1 = Note(Note.Params(36, 0.8, 20))
    n2 = Note(Note.Params(36, 0.8, 20))
    assert n1 == n2


def test_duration_json() -> None:
    assert Bar.json() == """{"num":1,"den":1}"""


def test_note_json() -> None:
    note = Note(
        Note.Params(
            note_num=60,
            velocity=0.9,
            dur_ms=20,
        ),
    )
    assert note.json() == """{"NoteEvent":{"note_num":60,"velocity":0.9,"dur_ms":20}}"""


def test_pattern_json() -> None:
    pattern = note_pattern("foo", "[c3]")
    assert (
        pattern.json()
        == """{"events":[{"action":{"NoteEvent":{"note_num":60,"velocity":0.8,"dur_ms":20}},"dur_frac":{"num":1,"den":1}}],"length_bars":{"num":1,"den":1},"name":"foo"}"""
    )


def test_pattern_add() -> None:
    note_pattern("foo", "[c3]") + note_pattern("bar", "[e3]") == (
        note_pattern("foo", "[c3 e3]") | resize(Bar * 2)
    )


def test_pattern_mul() -> None:
    foo = note_pattern("foo", "[c3]")
    assert foo * 3 == note_pattern("foo", "[c3 c3 c3]") | resize(Bar * 3)
