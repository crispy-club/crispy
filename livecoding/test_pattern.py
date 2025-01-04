import functools
import operator

import pytest

from livecoding.base_types import (
    Bar,
    Duration,
    Event,
    Half,
    Note,
    NotePattern,
    Quarter,
    Sixteenth,
    Zero,
)
from livecoding.pattern import (
    Perc,
    rev,
    rot,
    tran,
    lclip,
    rclip,
    ladd,
    radd,
    resize,
    name,
    revery,
    levery,
)
from livecoding.grammar import notes


def test_empty_pattern() -> None:
    assert notes("[]") | name("foo") == NotePattern(
        name="foo",
        length_bars=Bar,
        events=[],
    )


def test_pattern_with_velocity() -> None:
    assert notes("[c2,0.8 g2,0.7]") | name("foo") == NotePattern(
        name="foo",
        length_bars=Bar,
        events=[
            Event(
                action=Note(Note.Params(note_num=48, velocity=0.8, dur=Duration(1, 2))),
                dur=Bar / 2,
            ),
            Event(
                action=Note(Note.Params(note_num=55, velocity=0.7, dur=Duration(1, 2))),
                dur=Bar / 2,
            ),
        ],
    )


def test_pattern_with_rest() -> None:
    res = notes("[c2,0.8 ~]") | name("foo")
    assert res == NotePattern(
        name="foo",
        length_bars=Bar,
        events=[
            Event(
                action=Note(Note.Params(note_num=48, velocity=0.8, dur=Duration(1, 2))),
                dur=Bar / 2,
            ),
            Event(
                action="Rest",
                dur=Bar / 2,
            ),
        ],
    )


def test_pattern_with_rest_repeated() -> None:
    res = notes("[c2,0.8 ~2]") | name("foo")
    assert res == NotePattern(
        name="foo",
        length_bars=Bar,
        events=[
            Event(
                action=Note(Note.Params(note_num=48, velocity=0.8, dur=Duration(1, 2))),
                dur=Bar / 2,
            ),
            Event(
                action="Rest",
                dur=Bar / 4,
            ),
            Event(
                action="Rest",
                dur=Bar / 4,
            ),
        ],
    )


def test_add_sequence_of_patterns() -> None:
    res = functools.reduce(
        operator.add,
        [
            notes("[c2,0.8]"),
            notes("[e2,0.8]"),
            notes("[g2,0.8]"),
        ],
    ) | name("foo")
    assert res == (
        notes("[c2,0.8 e2,0.8 g2,0.8]") | resize(Duration(3, 1)) | name("foo")
    )


def test_pattern_with_note_multiplied() -> None:
    res = notes("[c2*4]") | name("foo")
    assert res == notes("[c2 c2 c2 c2]") | name("foo")


def test_rev() -> None:
    foo = notes("[c3 d3 e3 f3 g3]") | rev | name("foo")
    assert foo == notes("[g3 f3 e3 d3 c3]") | name("foo")


def test_transpose() -> None:
    assert notes("[c3 e3 g3]") | tran(7) | name("foo") == notes("[g3 b3 d4]") | name(
        "foo"
    )
    assert notes("[c4 ~ g4]") | tran(12) | name("foo") == notes("[c5 ~ g5]") | name(
        "foo"
    )


def test_rot_right() -> None:
    foo = notes("[c3 d3 e3 f3 g3]") | rot(1) | name("foo")
    assert foo == notes("[g3 c3 d3 e3 f3]") | name("foo")


def test_rot_left() -> None:
    foo = notes("[c3 d3 e3 f3 g3]") | rot(-2) | name("foo")
    assert foo == notes("[e3 f3 g3 c3 d3]") | name("foo")


def test_rclip() -> None:
    assert notes("[c3 d3 e3 f3 g3]", length_bars=Duration(5, 4)) | rclip(
        Bar / 4
    ) | name("foo") == notes("[c3 d3 e3 f3]", length_bars=Bar) | name("foo")

    assert notes("[c3 d3 e3 f3 g3]", length_bars=Duration(5, 4)) | rclip(
        Bar / 8
    ) | name("foo") == (
        notes("[c3 d3 e3 f3]", length_bars=Bar) + notes("[g3]", length_bars=(Bar / 8))
    ) | name("foo")

    assert notes("[c3 d3 e3 f3 g3]", length_bars=Duration(5, 4)) | rclip(
        Bar * Duration(3, 8)
    ) | name("foo") == (
        notes("[c3 d3 e3]", length_bars=Duration(3, 4))
        + notes("[f3]", length_bars=(Bar / 8))
    ) | name("foo")


def test_lclip() -> None:
    assert notes("[c3 d3 e3 f3 g3]", length_bars=Duration(5, 4)) | lclip(
        Bar / 4
    ) | name("foo") == notes("[d3 e3 f3 g3]", length_bars=Bar) | name("foo")

    assert notes("[c3 d3 e3 f3 g3]", length_bars=Duration(5, 4)) | lclip(
        Bar / 8
    ) | name("foo") == (
        notes("[c3]", length_bars=(Bar / 8)) + notes("[d3 e3 f3 g3]", length_bars=Bar)
    ) | name("foo")
    assert notes("[c3 d3 e3 f3 g3]", length_bars=Duration(5, 4)) | lclip(
        Bar * Duration(3, 8)
    ) | name("foo") == (
        notes("[d3]", length_bars=(Bar / 8))
        + notes("[e3 f3 g3]", length_bars=Duration(3, 4))
    ) | name("foo")


def test_lclip_zero() -> None:
    assert notes("[c3 d3 e3 f3 g3]", length_bars=Quarter * 5) | lclip(Zero) | name(
        "foo"
    ) == notes("[c3 d3 e3 f3 g3]", length_bars=Quarter * 5) | name("foo")


def test_ladd() -> None:
    assert notes("[c3 d3 e3 g3]") | ladd(notes("[c3 d3 e3 g3]") | tran(12)) | name(
        "foo"
    ) == notes("[c4 d4 e4 g4 c3 d3 e3 g3]", length_bars=Bar * 2) | name("foo")


def test_radd() -> None:
    assert notes("[c3 d3 e3 g3]") | radd(notes("[c3 d3 e3 g3]") | tran(12)) | name(
        "foo"
    ) == notes("[c3 d3 e3 g3 c4 d4 e4 g4]", length_bars=Bar * 2) | name("foo")


def test_resize() -> None:
    assert notes("[c3 d3 e3 g3]") | resize(Bar * 3) | name("foo") == notes(
        "[c3 d3 e3 g3]", length_bars=Bar * 3
    ) | name("foo")
    assert notes("[c3 d3 e3 g3]") | resize(Bar / 16) | name("foo") == notes(
        "[c3 d3 e3 g3]", length_bars=Bar / 16
    ) | name("foo")


def test_revery() -> None:
    assert notes("[c3 d3 e3 g3]") | revery(2, rev) | name("foo") == notes(
        "[c3 d3 e3 g3]"
    ) + notes("[g3 e3 d3 c3]") | name("foo")


def test_levery() -> None:
    assert notes("[c3 d3 e3 g3]") | levery(2, rev) | name("foo") == notes(
        "[g3 e3 d3 c3]"
    ) + notes("[c3 d3 e3 g3]") | name("foo")


def test_perc_pattern_single_lane() -> None:
    perc = Perc()
    assert perc.parse("c1 = X.") == [
        NotePattern(
            name="c1",
            length_bars=Bar / 8,
            events=[
                Event(
                    action=Note(
                        Note.Params(
                            note_num=36,
                            velocity=0.9,
                            dur=Duration(1, 2),
                        )
                    ),
                    dur=Duration(1, 16),
                ),
                Event(
                    action="Rest",
                    dur=Duration(1, 16),
                ),
            ],
        ),
    ]


def test_perc_pattern_tie() -> None:
    perc = Perc()
    assert perc.parse("c1 = X++x.") == [
        NotePattern(
            name="c1",
            length_bars=Sixteenth * 5,
            events=[
                Event(
                    action=Note(
                        Note.Params(
                            note_num=36,
                            velocity=0.9,
                            dur=Half,
                        )
                    ),
                    dur=Sixteenth * 3,
                ),
                Event(
                    action=Note(
                        Note.Params(
                            note_num=36,
                            velocity=0.4,
                            dur=Half,
                        )
                    ),
                    dur=Sixteenth,
                ),
                Event(
                    action="Rest",
                    dur=Sixteenth,
                ),
            ],
        ),
    ]


def test_perc_empty_pattern() -> None:
    perc = Perc()
    assert perc.parse("c1 = ") == [
        NotePattern(
            name="c1",
            length_bars=Zero,
            events=[],
        ),
    ]


def test_perc_broken_notation() -> None:
    perc = Perc()

    with pytest.raises(ValueError) as ex:
        perc.parse("c1 = foo")

    assert str(ex.value) == "unsupported notation: f (line format is NOTE = [Xx_.])"
