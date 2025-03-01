import pytest

from crispy.base_types import Bar, Event, Half, Note, PluginPattern
from crispy.filters import name
from crispy.pat import (
    InvalidSyntaxError,
    _Group,
    _get_subgroups_r as get_subgroups_r,
    _separate_delimiters as separate_delimiters,
    get_velocity,
    pat,
)


def test_separate_delimiters_empty_pattern() -> None:
    assert separate_delimiters(["[]"]) == ["[", "]"]


def test_separate_delimiters_no_nesting() -> None:
    assert separate_delimiters(["[C", "E", "G]"]) == ["[", "C", "E", "G", "]"]


def test_separate_delimiters_with_nesting() -> None:
    assert separate_delimiters(["[C", "[E", "A]", "G]"]) == [
        "[",
        "C",
        "[",
        "E",
        "A",
        "]",
        "G",
        "]",
    ]
    assert separate_delimiters(["[C'2y", "[E3x", "A]", "G]"]) == [
        "[",
        "C'2y",
        "[",
        "E3x",
        "A",
        "]",
        "G",
        "]",
    ]


def test_get_subgroups_r_empty_pattern() -> None:
    group = _Group(children=[])
    get_subgroups_r(group, ["[]"])
    assert group == _Group(children=[])


def test_get_subgroups_r_single_note() -> None:
    group = _Group(children=[])
    get_subgroups_r(group, ["C"])
    assert group == _Group(children=["C"])


def test_get_subgroups_r_three_notes() -> None:
    group = _Group(children=[])
    get_subgroups_r(group, ["C", "E", "G"])
    assert group == _Group(children=["C", "E", "G"])


def test_get_subgroups_r_nested() -> None:
    group = _Group(children=[])
    get_subgroups_r(group, ["C", "[", "E", "A", "]", "G"])
    assert group == _Group(children=["C", _Group(children=["E", "A"]), "G"])


def test_pat_empty_pattern() -> None:
    empty = PluginPattern(
        events=[],
        length_bars=Bar,
        name="foo",
    )
    pattern = pat("[]") | name("foo")
    assert pattern == empty


def test_pat_single_opening_bracket() -> None:
    with pytest.raises(InvalidSyntaxError):
        _ = pat("[")


def test_pat_one_note() -> None:
    expect = PluginPattern(
        events=[
            Event(
                dur=Bar,
                action=Note(Note.Params(note_num=60, velocity=0.58, dur=Half)),
            ),
        ],
        length_bars=Bar,
        name="foo",
    )
    pattern = pat("[C]") | name("foo")
    assert pattern == expect, str(pattern)


def test_pat_three_notes() -> None:
    expect = PluginPattern(
        events=[
            Event(
                dur=Bar / 3,
                action=Note(Note.Params(note_num=60, velocity=0.58, dur=Half)),
            ),
            Event(
                dur=Bar / 3,
                action=Note(Note.Params(note_num=64, velocity=0.58, dur=Half)),
            ),
            Event(
                dur=Bar / 3,
                action=Note(Note.Params(note_num=67, velocity=0.58, dur=Half)),
            ),
        ],
        length_bars=Bar,
        name="foo",
    )
    pattern = pat("[C E G]") | name("foo")
    assert pattern == expect, str(pattern)


def test_pat_nested_groups() -> None:
    expect = PluginPattern(
        events=[
            Event(
                dur=Bar / 2,
                action=Note(Note.Params(note_num=60, velocity=0.58, dur=Half)),
            ),
            Event(
                dur=Bar / 4,
                action=Note(Note.Params(note_num=64, velocity=0.58, dur=Half)),
            ),
            Event(
                dur=Bar / 4,
                action=Note(Note.Params(note_num=67, velocity=0.58, dur=Half)),
            ),
        ],
        length_bars=Bar,
        name="foo",
    )
    pattern = pat("[C [E G]]") | name("foo")
    assert pattern == expect, str(pattern)


def test_pat_single_velocity_value() -> None:
    pattern = pat("[a]") | name("foo")
    assert pattern == PluginPattern(
        events=[
            Event(
                action=Note(Note.Params(velocity=0.04, dur=Half, note_num=60)), dur=Bar
            )
        ],
        length_bars=Bar,
        name="foo",
    )


def test_pat_single_pitch_class_value() -> None:
    pattern = pat("[C]") | name("foo")
    assert pattern == PluginPattern(
        events=[
            Event(
                action=Note(Note.Params(velocity=0.58, dur=Half, note_num=60)), dur=Bar
            )
        ],
        length_bars=Bar,
        name="foo",
    )


def test_pat_single_pitch_class_with_octave() -> None:
    pattern = pat("[C1]") | name("foo")
    assert pattern == PluginPattern(
        events=[
            Event(
                action=Note(Note.Params(velocity=0.58, dur=Half, note_num=36)), dur=Bar
            )
        ],
        length_bars=Bar,
        name="foo",
    )


def test_pat_single_sharp_pitch_class_with_octave() -> None:
    pattern = pat("[C'1]") | name("foo")
    assert pattern == PluginPattern(
        events=[
            Event(
                action=Note(Note.Params(velocity=0.58, dur=Half, note_num=37)), dur=Bar
            )
        ],
        length_bars=Bar,
        name="foo",
    )


def test_pat_single_pitch_class_sharp_with_octave_and_velocity() -> None:
    pattern = pat("[C'1w]") | name("foo")
    assert pattern == PluginPattern(
        events=[
            Event(
                action=Note(Note.Params(velocity=0.88, dur=Half, note_num=37)), dur=Bar
            )
        ],
        length_bars=Bar,
        name="foo",
    )


def test_pat_two_velocities() -> None:
    pattern = pat("[w x]") | name("foo")
    assert pattern == PluginPattern(
        events=[
            Event(
                action=Note(Note.Params(velocity=0.88, dur=Half, note_num=60)),
                dur=Bar / 2,
            ),
            Event(
                action=Note(Note.Params(velocity=0.92, dur=Half, note_num=60)),
                dur=Bar / 2,
            ),
        ],
        length_bars=Bar,
        name="foo",
    )


def test_pat_two_notes() -> None:
    pattern = pat("[Cw Dx]") | name("foo")
    assert pattern == PluginPattern(
        events=[
            Event(
                action=Note(Note.Params(velocity=0.88, dur=Half, note_num=60)),
                dur=Bar / 2,
            ),
            Event(
                action=Note(Note.Params(velocity=0.92, dur=Half, note_num=62)),
                dur=Bar / 2,
            ),
        ],
        length_bars=Bar,
        name="foo",
    )


def test_pat_grammar_pattern_with_rests_repeated_grouping() -> None:
    pattern = pat("[y .;2]") | name("foo")
    assert pattern == PluginPattern(
        length_bars=Bar,
        events=[
            Event(
                action=Note(Note.Params(note_num=60, velocity=0.96, dur=Half)),
                dur=Bar / 2,
            ),
            Event(action="Rest", dur=Bar / 4),
            Event(action="Rest", dur=Bar / 4),
        ],
        name="foo",
    )


def test_pat_grammar_pattern_with_rests_repeated_without_grouping() -> None:
    pattern = pat("[y .:2]") | name("foo")
    assert pattern == PluginPattern(
        length_bars=Bar,
        events=[
            Event(
                action=Note(Note.Params(note_num=60, velocity=0.96, dur=Half)),
                dur=Bar / 3,
            ),
            Event(action="Rest", dur=Bar / 3),
            Event(action="Rest", dur=Bar / 3),
        ],
        name="foo",
    )


def test_pat_grammar_patterns_with_ties() -> None:
    pattern = pat("[Cy _:3 Gw _]") | name("foo")
    assert pattern == PluginPattern(
        length_bars=Bar,
        events=[
            Event(
                action=Note(Note.Params(note_num=60, velocity=0.96, dur=Half)),
                dur=(Bar / 3) * 2,
            ),
            Event(
                action=Note(Note.Params(note_num=67, velocity=0.88, dur=Half)),
                dur=Bar / 3,
            ),
        ],
        name="foo",
    )


def test_pat_grammar_patterns_with_tie_sugar() -> None:
    pattern = pat("[Cy@4 Gw _]") | name("foo")
    assert pattern == PluginPattern(
        length_bars=Bar,
        events=[
            Event(
                action=Note(Note.Params(note_num=60, velocity=0.96, dur=Half)),
                dur=(Bar / 3) * 2,
            ),
            Event(
                action=Note(Note.Params(note_num=67, velocity=0.88, dur=Half)),
                dur=Bar / 3,
            ),
        ],
        name="foo",
    )


def test_pat_grammar_patterns_with_alternation1() -> None:
    pattern = pat("[Cy <Gw Ex>]") | name("foo")
    assert pattern == PluginPattern(
        length_bars=Bar,
        events=[
            Event(
                action=Note(Note.Params(note_num=60, velocity=0.96, dur=Half)),
                dur=Bar / 4,
            ),
            Event(
                action=Note(Note.Params(note_num=67, velocity=0.88, dur=Half)),
                dur=Bar / 4,
            ),
            Event(
                action=Note(Note.Params(note_num=60, velocity=0.96, dur=Half)),
                dur=Bar / 4,
            ),
            Event(
                action=Note(Note.Params(note_num=64, velocity=0.92, dur=Half)),
                dur=Bar / 4,
            ),
        ],
        name="foo",
    )


def test_pat_grammar_patterns_with_alternation2() -> None:
    pattern = pat("[Cy <Gw Ex <Fd Ap>>]") | name("foo")
    print(pattern)
    assert pattern == PluginPattern(
        length_bars=Bar,
        events=[
            Event(
                action=Note(
                    Note.Params(note_num=60, velocity=get_velocity("y"), dur=Half)
                ),
                dur=Bar / 10,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=67, velocity=get_velocity("w"), dur=Half)
                ),
                dur=Bar / 10,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=60, velocity=get_velocity("y"), dur=Half)
                ),
                dur=Bar / 10,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=64, velocity=get_velocity("x"), dur=Half)
                ),
                dur=Bar / 10,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=60, velocity=get_velocity("y"), dur=Half)
                ),
                dur=Bar / 10,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=65, velocity=get_velocity("d"), dur=Half)
                ),
                dur=Bar / 10,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=60, velocity=get_velocity("y"), dur=Half)
                ),
                dur=Bar / 10,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=64, velocity=get_velocity("x"), dur=Half)
                ),
                dur=Bar / 10,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=60, velocity=get_velocity("y"), dur=Half)
                ),
                dur=Bar / 10,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=69, velocity=get_velocity("p"), dur=Half)
                ),
                dur=Bar / 10,
            ),
        ],
        name="foo",
    )


def test_pat_grammar_patterns_with_alternation3() -> None:
    pattern = pat("[Cy <[Gw Ex] <Fd Ap>>]") | name("foo")
    print(pattern)
    assert pattern == PluginPattern(
        length_bars=Bar,
        events=[
            Event(
                action=Note(
                    Note.Params(note_num=60, velocity=get_velocity("y"), dur=Half)
                ),
                dur=Bar / 8,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=67, velocity=get_velocity("w"), dur=Half)
                ),
                dur=Bar / 16,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=64, velocity=get_velocity("x"), dur=Half)
                ),
                dur=Bar / 16,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=60, velocity=get_velocity("y"), dur=Half)
                ),
                dur=Bar / 8,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=65, velocity=get_velocity("d"), dur=Half)
                ),
                dur=Bar / 8,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=60, velocity=get_velocity("y"), dur=Half)
                ),
                dur=Bar / 8,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=67, velocity=get_velocity("w"), dur=Half)
                ),
                dur=Bar / 16,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=64, velocity=get_velocity("x"), dur=Half)
                ),
                dur=Bar / 16,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=60, velocity=get_velocity("y"), dur=Half)
                ),
                dur=Bar / 8,
            ),
            Event(
                action=Note(
                    Note.Params(note_num=69, velocity=get_velocity("p"), dur=Half)
                ),
                dur=Bar / 8,
            ),
        ],
        name="foo",
    )
