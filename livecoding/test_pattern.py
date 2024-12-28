import functools
import operator

from livecoding.base_types import Bar, Duration, Event, Note, NotePattern
from livecoding.pattern import rev, rot, tran, resize
from livecoding.plugin import note_pattern


def test_empty_pattern() -> None:
    assert note_pattern("foo", "[]") == NotePattern(
        name="foo",
        length_bars=Bar,
        events=[],
    )


def test_pattern_with_velocity() -> None:
    assert note_pattern("foo", "[c2,0.8 g2,0.7]") == NotePattern(
        name="foo",
        length_bars=Bar,
        events=[
            Event(
                action=Note(Note.Params(note_num=48, velocity=0.8, dur_ms=20)),
                dur_frac=Bar / 2,
            ),
            Event(
                action=Note(Note.Params(note_num=55, velocity=0.7, dur_ms=20)),
                dur_frac=Bar / 2,
            ),
        ],
    )


def test_pattern_with_rest() -> None:
    res = note_pattern("foo", "[c2,0.8 .]")
    assert res == NotePattern(
        name="foo",
        length_bars=Bar,
        events=[
            Event(
                action=Note(Note.Params(note_num=48, velocity=0.8, dur_ms=20)),
                dur_frac=Bar / 2,
            ),
            Event(
                action="Rest",
                dur_frac=Bar / 2,
            ),
        ],
    )


def test_add_sequence_of_patterns() -> None:
    res = functools.reduce(
        operator.add,
        [
            note_pattern("p1", "[c2,0.8]"),
            note_pattern("p2", "[e2,0.8]"),
            note_pattern("p3", "[g2,0.8]"),
        ],
    )
    assert res == (
        note_pattern("p1", "[c2,0.8 e2,0.8 g2,0.8]") | resize(Duration(3, 1))
    )


def test_rev() -> None:
    foo = note_pattern("foo", "[c3 d3 e3 f3 g3]") | rev
    assert foo == note_pattern("foo", "[g3 f3 e3 d3 c3]")


def test_transpose() -> None:
    foo = note_pattern("foo", "[c3 e3 g3]") | tran(7)
    assert foo == note_pattern("foo", "[g3 b3 d4]")


def test_rot_right() -> None:
    foo = note_pattern("foo", "[c3 d3 e3 f3 g3]") | rot(1)
    assert foo == note_pattern("foo", "[g3 c3 d3 e3 f3]")


def test_rot_left() -> None:
    foo = note_pattern("foo", "[c3 d3 e3 f3 g3]") | rot(-2)
    assert foo == note_pattern("foo", "[e3 f3 g3 c3 d3]")
