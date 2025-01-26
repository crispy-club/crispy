import pytest

from livecoding.base_types import Bar, Event, Half, Note, PluginPattern
from livecoding.pattern import name

# from livecoding.scales_v2 import _SUBGROUPS_REGEX as subgroups_regex, rhythm_v2
from livecoding.scales_v2 import (
    InvalidSyntaxError,
    _Group,
    _get_subgroups_r as get_subgroups_r,
    _separate_delimiters as separate_delimiters,
    rhythm_v2,
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


def test_rhythm_v2_empty_pattern() -> None:
    empty = PluginPattern(
        events=[],
        length_bars=Bar,
        name="foo",
    )
    pattern = rhythm_v2("[]") | name("foo")
    assert pattern == empty


def test_rhythm_v2_single_opening_bracket() -> None:
    with pytest.raises(InvalidSyntaxError):
        _ = rhythm_v2("[")


def test_rhythm_v2_one_note() -> None:
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
    pattern = rhythm_v2("[C]") | name("foo")
    assert pattern == expect, str(pattern)


def test_rhythm_v2_three_notes() -> None:
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
    pattern = rhythm_v2("[C E G]") | name("foo")
    assert pattern == expect, str(pattern)


def test_rhythm_v2_nested_groups() -> None:
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
    pattern = rhythm_v2("[C [E G]]") | name("foo")
    assert pattern == expect, str(pattern)


def test_rhythm_v2_single_velocity_value() -> None:
    pattern = rhythm_v2("[a]") | name("foo")
    assert pattern == PluginPattern(
        events=[
            Event(
                action=Note(Note.Params(velocity=0.04, dur=Half, note_num=60)), dur=Bar
            )
        ],
        length_bars=Bar,
        name="foo",
    )


def test_rhythm_v2_single_pitch_class_value() -> None:
    pattern = rhythm_v2("[C]") | name("foo")
    assert pattern == PluginPattern(
        events=[
            Event(
                action=Note(Note.Params(velocity=0.58, dur=Half, note_num=60)), dur=Bar
            )
        ],
        length_bars=Bar,
        name="foo",
    )


def test_rhythm_v2_single_pitch_class_with_octave() -> None:
    pattern = rhythm_v2("[C1]") | name("foo")
    assert pattern == PluginPattern(
        events=[
            Event(
                action=Note(Note.Params(velocity=0.58, dur=Half, note_num=36)), dur=Bar
            )
        ],
        length_bars=Bar,
        name="foo",
    )


def test_rhythm_v2_single_sharp_pitch_class_with_octave() -> None:
    pattern = rhythm_v2("[C'1]") | name("foo")
    assert pattern == PluginPattern(
        events=[
            Event(
                action=Note(Note.Params(velocity=0.58, dur=Half, note_num=37)), dur=Bar
            )
        ],
        length_bars=Bar,
        name="foo",
    )


def test_rhythm_v2_single_pitch_class_sharp_with_octave_and_velocity() -> None:
    pattern = rhythm_v2("[C'1w]") | name("foo")
    assert pattern == PluginPattern(
        events=[
            Event(
                action=Note(Note.Params(velocity=0.88, dur=Half, note_num=37)), dur=Bar
            )
        ],
        length_bars=Bar,
        name="foo",
    )


def test_rhythm_v2_two_velocities() -> None:
    pattern = rhythm_v2("[w x]") | name("foo")
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


def test_rhythm_v2_two_notes() -> None:
    pattern = rhythm_v2("[Cw Dx]") | name("foo")
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


def test_rhythm_v2_grammar_pattern_with_rests_repeated_grouping() -> None:
    pattern = rhythm_v2("[y .;2]") | name("foo")
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


def test_rhythm_v2_grammar_pattern_with_rests_repeated_without_grouping() -> None:
    pattern = rhythm_v2("[y .:2]") | name("foo")
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


def test_rhythm_v2_grammar_patterns_with_ties() -> None:
    pattern = rhythm_v2("[Cy _:3 Gw _]") | name("foo")
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


def test_rhythm_v2_grammar_patterns_with_tie_sugar() -> None:
    pattern = rhythm_v2("[Cy@4 Gw _]") | name("foo")
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


# def test_rhythm_v2_grammar_patterns_with_alternation() -> None:
#     pattern = rhythm_v2("[Cy <Gw Ex>]") | name("foo")
#     assert pattern == PluginPattern(
#         length_bars=Bar,
#         events=[
#             Event(
#                 action=Note(Note.Params(note_num=60, velocity=0.96, dur=Half)),
#                 dur=(Bar / 3) * 2,
#             ),
#             Event(
#                 action=Note(Note.Params(note_num=67, velocity=0.88, dur=Half)),
#                 dur=Bar / 3,
#             ),
#         ],
#         name="foo",
#     )
