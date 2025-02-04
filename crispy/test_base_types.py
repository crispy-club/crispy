from crispy.base_types import Bar, Duration, Event, Note
from crispy.filters import name, resize
from crispy.pat import pat


def test_duration_add() -> None:
    assert Duration(1, 5) + Duration(1, 3) == Duration(8, 15)


def test_duration_division() -> None:
    assert Bar / 4 == Duration(1, 4)
    half_note = Bar / 2
    quarter_note = Bar / 4
    assert half_note / quarter_note == Bar * 2
    assert 1 / quarter_note == Bar * 4


def test_duration_eq() -> None:
    assert Duration(1, 5) == Duration(1, 5)
    assert Duration(1, 5) != "1/5"


def test_duration_gt() -> None:
    assert Duration(1, 2) > Duration(1, 4)
    assert Duration(1, 5) >= Duration(1, 5)


def test_duration_lt() -> None:
    assert Duration(1, 5) < Duration(1, 4)
    assert Duration(1, 5) <= Duration(1, 5)


def test_duration_subtraction() -> None:
    assert Duration(1, 4) - Duration(1, 8) == Duration(1, 8)


def test_duration_str() -> None:
    assert str(Duration(1, 4)) == "1/4"


def test_event_json() -> None:
    ev = Event(
        action=Note(Note.Params(note_num=60, velocity=0.7, dur=Duration(1, 2))),
        dur=Duration(1, 4),
    )
    assert (
        ev.json()
        == """{"action":{"NoteEvent":{"note_num":60,"velocity":0.7,"dur":{"num":1,"den":2}}},"dur":{"num":1,"den":4}}"""
    )


def test_note_eq() -> None:
    n1 = Note(Note.Params(36, 0.8, Duration(1, 4)))
    n2 = Note(Note.Params(36, 0.8, Duration(1, 4)))
    assert n1 == n2


def test_duration_json() -> None:
    assert Bar.json() == """{"num":1,"den":1}"""


def test_note_json() -> None:
    note = Note(
        Note.Params(
            note_num=60,
            velocity=0.9,
            dur=Duration(1, 2),
        ),
    )
    assert (
        note.json()
        == """{"NoteEvent":{"note_num":60,"velocity":0.9,"dur":{"num":1,"den":2}}}"""
    )


def test_note_pattern_json() -> None:
    pattern = pat("[C3]") | name("foo")
    assert (
        pattern.json()
        == """{"name":"foo","events":[{"action":{"NoteEvent":{"note_num":60,"velocity":0.58,"dur":{"num":1,"den":2}}},"dur":{"num":1,"den":1}}],"length_bars":{"num":1,"den":1}}"""
    )


def test_pattern_add() -> None:
    (pat("[c3]") + pat("[e3]")) | name("foo") == (
        pat("[c3 e3]") | resize(Bar * 2) | name("foo")
    )


def test_pattern_mul() -> None:
    foo = pat("[c3]") | name("foo")
    assert foo * 3 == (pat("[c3 c3 c3]") | resize(Bar * 3) | name("foo"))
