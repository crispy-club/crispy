from livecoding.base_types import Bar, Event, Half, Note, PluginPattern

# from livecoding.notes_grammar import notes
from livecoding.pattern import name

# from livecoding.pitches import C, Octave
# from livecoding.scales import Maj, cycle
from livecoding.scales_v2 import rhythm_v2


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


def test_rhythm_v2_grammar_pattern_with_rests_repeated_grouping() -> None:
    pattern = rhythm_v2("[y.;2]") | name("foo")
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
    pattern = rhythm_v2("[y.:2]") | name("foo")
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
