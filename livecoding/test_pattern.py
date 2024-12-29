import functools
import operator

from livecoding.base_types import Bar, Duration, Event, Note, NotePattern
from livecoding.pattern import rev, rot, tran, lclip, rclip, ladd, radd, resize, name
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


def test_pattern_with_note_multiplied() -> None:
    res = note_pattern("p1", "[c2*4]")
    assert res == note_pattern("p1", "[c2 c2 c2 c2]")


def test_rev() -> None:
    foo = note_pattern("foo", "[c3 d3 e3 f3 g3]") | rev
    assert foo == note_pattern("foo", "[g3 f3 e3 d3 c3]")


def test_transpose() -> None:
    assert note_pattern("foo", "[c3 e3 g3]") | tran(7) == note_pattern(
        "foo", "[g3 b3 d4]"
    )
    assert note_pattern("foo", "[c4 . g4]") | tran(12) == note_pattern(
        "foo", "[c5 . g5]"
    )


def test_rot_right() -> None:
    foo = note_pattern("foo", "[c3 d3 e3 f3 g3]") | rot(1)
    assert foo == note_pattern("foo", "[g3 c3 d3 e3 f3]")


def test_rot_left() -> None:
    foo = note_pattern("foo", "[c3 d3 e3 f3 g3]") | rot(-2)
    assert foo == note_pattern("foo", "[e3 f3 g3 c3 d3]")


def test_rclip() -> None:
    assert note_pattern("foo", "[c3 d3 e3 f3 g3]", length_bars=Duration(5, 4)) | rclip(
        Bar / 4
    ) == note_pattern("foo", "[c3 d3 e3 f3]", length_bars=Bar)

    assert note_pattern("foo", "[c3 d3 e3 f3 g3]", length_bars=Duration(5, 4)) | rclip(
        Bar / 8
    ) == (
        note_pattern("foo", "[c3 d3 e3 f3]", length_bars=Bar)
        + note_pattern("foo", "[g3]", length_bars=(Bar / 8))
    )

    assert note_pattern("foo", "[c3 d3 e3 f3 g3]", length_bars=Duration(5, 4)) | rclip(
        Bar * Duration(3, 8)
    ) == (
        note_pattern("foo", "[c3 d3 e3]", length_bars=Duration(3, 4))
        + note_pattern("foo", "[f3]", length_bars=(Bar / 8))
    )


def test_lclip() -> None:
    assert note_pattern("foo", "[c3 d3 e3 f3 g3]", length_bars=Duration(5, 4)) | lclip(
        Bar / 4
    ) == note_pattern("foo", "[d3 e3 f3 g3]", length_bars=Bar)

    assert note_pattern("foo", "[c3 d3 e3 f3 g3]", length_bars=Duration(5, 4)) | lclip(
        Bar / 8
    ) == (
        note_pattern("foo", "[c3]", length_bars=(Bar / 8))
        + note_pattern("foo", "[d3 e3 f3 g3]", length_bars=Bar)
    )

    assert note_pattern("foo", "[c3 d3 e3 f3 g3]", length_bars=Duration(5, 4)) | lclip(
        Bar * Duration(3, 8)
    ) == (
        note_pattern("foo", "[d3]", length_bars=(Bar / 8))
        + note_pattern("foo", "[e3 f3 g3]", length_bars=Duration(3, 4))
    )


def test_ladd() -> None:
    assert note_pattern("foo", "[c3 d3 e3 g3]") | ladd(
        note_pattern("foo", "[c3 d3 e3 g3]") | tran(12)
    ) == note_pattern("foo", "[c4 d4 e4 g4 c3 d3 e3 g3]", length_bars=Bar * 2)


def test_radd() -> None:
    assert note_pattern("foo", "[c3 d3 e3 g3]") | radd(
        note_pattern("foo", "[c3 d3 e3 g3]") | tran(12)
    ) == note_pattern("foo", "[c3 d3 e3 g3 c4 d4 e4 g4]", length_bars=Bar * 2)


def test_resize() -> None:
    assert note_pattern("foo", "[c3 d3 e3 g3]") | resize(Bar * 3) == note_pattern(
        "foo", "[c3 d3 e3 g3]", length_bars=Bar * 3
    )
    assert note_pattern("foo", "[c3 d3 e3 g3]") | resize(Bar / 16) == note_pattern(
        "foo", "[c3 d3 e3 g3]", length_bars=Bar / 16
    )


def test_name() -> None:
    assert note_pattern("foo", "[c3 d3 e3 g3]") | name("baz") == note_pattern(
        "baz", "[c3 d3 e3 g3]"
    )
