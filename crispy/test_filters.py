import functools
import operator

import pytest

from crispy.base_types import (
    Bar,
    Duration,
    Event,
    Half,
    Note,
    PluginPattern,
    Quarter,
    Sixteenth,
    Zero,
)
from crispy.filters import (
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
    each,
    each_note,
)
from crispy.pat import pat
from crispy.perc import Perc


def test_empty_pattern() -> None:
    assert pat("[]") | name("foo") == PluginPattern(
        name="foo",
        length_bars=Bar,
        events=[],
    )


def test_add_sequence_of_patterns() -> None:
    res = functools.reduce(
        operator.add,
        [
            pat("[C2x]"),
            pat("[E2x]"),
            pat("[G2x]"),
        ],
    ) | name("foo")
    assert res == (pat("[C2x E2x G2x]") | resize(Duration(3, 1)) | name("foo"))


def test_rev() -> None:
    foo = pat("[C3 D3 E3 F3 G3]") | rev | name("foo")
    assert foo == pat("[G3 F3 E3 D3 C3]") | name("foo")


def test_transpose() -> None:
    assert pat("[C3 E3 G3]") | tran(7) | name("foo") == pat("[G3 B3 D4]") | name("foo")
    assert pat("[C4 . G4]") | tran(12) | name("foo") == pat("[C5 . G5]") | name("foo")


def test_rot_right() -> None:
    foo = pat("[C3 D3 E3 F3 G3]") | rot(1) | name("foo")
    assert foo == pat("[G3 C3 D3 E3 F3]") | name("foo")


def test_rot_left() -> None:
    foo = pat("[C3 D3 E3 F3 G3]") | rot(-2) | name("foo")
    assert foo == pat("[E3 F3 G3 C3 D3]") | name("foo")


def test_rclip() -> None:
    assert pat("[C3 D3 E3 F3 G3]", length_bars=Duration(5, 4)) | rclip(Bar / 4) | name(
        "foo"
    ) == pat("[C3 D3 E3 F3]", length_bars=Bar) | name("foo")

    assert pat("[C3 D3 E3 F3 G3]", length_bars=Duration(5, 4)) | rclip(Bar / 8) | name(
        "foo"
    ) == (
        pat("[C3 D3 E3 F3]", length_bars=Bar) + pat("[G3]", length_bars=(Bar / 8))
    ) | name("foo")

    assert pat("[C3 D3 E3 F3 G3]", length_bars=Duration(5, 4)) | rclip(
        Bar * Duration(3, 8)
    ) | name("foo") == (
        pat("[C3 D3 E3]", length_bars=Duration(3, 4))
        + pat("[F3]", length_bars=(Bar / 8))
    ) | name("foo")


def test_lclip() -> None:
    assert pat("[C3 D3 E3 F3 G3]", length_bars=Duration(5, 4)) | lclip(Bar / 4) | name(
        "foo"
    ) == pat("[D3 E3 F3 G3]", length_bars=Bar) | name("foo")

    assert pat("[C3 D3 E3 F3 G3]", length_bars=Duration(5, 4)) | lclip(Bar / 8) | name(
        "foo"
    ) == (
        pat("[C3]", length_bars=(Bar / 8)) + pat("[D3 E3 F3 G3]", length_bars=Bar)
    ) | name("foo")
    assert pat("[C3 D3 E3 F3 G3]", length_bars=Duration(5, 4)) | lclip(
        Bar * Duration(3, 8)
    ) | name("foo") == (
        pat("[D3]", length_bars=(Bar / 8))
        + pat("[E3 F3 G3]", length_bars=Duration(3, 4))
    ) | name("foo")


def test_lclip_zero() -> None:
    assert pat("[C3 D3 E3 F3 G3]", length_bars=Quarter * 5) | lclip(Zero) | name(
        "foo"
    ) == pat("[C3 D3 E3 F3 G3]", length_bars=Quarter * 5) | name("foo")


def test_ladd() -> None:
    assert pat("[C3 D3 E3 G3]") | ladd(pat("[C3 D3 E3 G3]") | tran(12)) | name(
        "foo"
    ) == pat("[C4 D4 E4 G4 C3 D3 E3 G3]", length_bars=Bar * 2) | name("foo")


def test_radd() -> None:
    assert pat("[C3 D3 E3 G3]") | radd(pat("[C3 D3 E3 G3]") | tran(12)) | name(
        "foo"
    ) == pat("[C3 D3 E3 G3 C4 D4 E4 G4]", length_bars=Bar * 2) | name("foo")


def test_resize() -> None:
    assert pat("[C3 D3 E3 G3]") | resize(Bar * 3) | name("foo") == pat(
        "[C3 D3 E3 G3]", length_bars=Bar * 3
    ) | name("foo")
    assert pat("[C3 D3 E3 G3]") | resize(Bar / 16) | name("foo") == pat(
        "[C3 D3 E3 G3]", length_bars=Bar / 16
    ) | name("foo")


def test_revery() -> None:
    assert pat("[C3 D3 E3 G3]") | revery(2, rev) | name("foo") == pat(
        "[C3 D3 E3 G3]"
    ) + pat("[G3 E3 D3 C3]") | name("foo")


def test_levery() -> None:
    assert pat("[C3 D3 E3 G3]") | levery(2, rev) | name("foo") == pat(
        "[G3 E3 D3 C3]"
    ) + pat("[C3 D3 E3 G3]") | name("foo")


def test_perc_pattern_single_lane() -> None:
    perc = Perc()
    assert perc.parse("c1 = X.") == [
        PluginPattern(
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
        PluginPattern(
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
        PluginPattern(
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


def test_each_identity() -> None:
    assert pat("[C3 D3 E3 G3]") | each(lambda e: e) | name("foo") == pat(
        "[C3 D3 E3 G3]"
    ) | name("foo")


def test_each_multiply_duration() -> None:
    assert pat("[C3 G3]") | each(lambda e: e * Duration(2, 1)) | name(
        "foo"
    ) == PluginPattern(
        name="foo",
        length_bars=Bar * 2,
        events=[
            Event(
                dur=Bar,
                action=Note(
                    Note.Params(
                        note_num=60,
                        velocity=0.58,
                        dur=Bar,
                    ),
                ),
            ),
            Event(
                dur=Bar,
                action=Note(
                    Note.Params(
                        note_num=67,
                        velocity=0.58,
                        dur=Bar,
                    ),
                ),
            ),
        ],
    )


def test_each_note_multiply_duration() -> None:
    assert pat("[C3 G3]") | each_note(lambda n: n * Duration(4, 1)) | name(
        "foo"
    ) == PluginPattern(
        name="foo",
        length_bars=Bar,
        events=[
            Event(
                dur=Bar / 2,
                action=Note(
                    Note.Params(
                        note_num=60,
                        velocity=0.58,
                        dur=Duration(2, 1),
                    ),
                ),
            ),
            Event(
                dur=Bar / 2,
                action=Note(
                    Note.Params(
                        note_num=67,
                        velocity=0.58,
                        dur=Duration(2, 1),
                    ),
                ),
            ),
        ],
    )


def test_each_note_set_duration() -> None:
    assert pat("[C3 G3]") | each_note(lambda n: n.set_dur(Duration(2, 1))) | name(
        "foo"
    ) == PluginPattern(
        name="foo",
        length_bars=Bar,
        events=[
            Event(
                dur=Bar / 2,
                action=Note(
                    Note.Params(
                        note_num=60,
                        velocity=0.58,
                        dur=Duration(2, 1),
                    ),
                ),
            ),
            Event(
                dur=Bar / 2,
                action=Note(
                    Note.Params(
                        note_num=67,
                        velocity=0.58,
                        dur=Duration(2, 1),
                    ),
                ),
            ),
        ],
    )
