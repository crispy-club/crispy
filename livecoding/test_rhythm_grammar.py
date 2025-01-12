from livecoding.base_types import Bar
from livecoding.notes_grammar import notes
from livecoding.pattern import name
from livecoding.pitches import C, Octave
from livecoding.rhythm_grammar import Hit, Rhythm, rhythm
from livecoding.scales import Maj, cycle


def test_rhythm_grammar_empty_pattern() -> None:
    assert rhythm("[]") == Rhythm(hits=[], length_bars=Bar)


def test_rhythm_grammar_one_hit() -> None:
    assert rhythm("[0.9]") == Rhythm(hits=[Hit(velocity=0.9, dur=Bar)], length_bars=Bar)


def test_rhythm_grammar_nested_groups() -> None:
    assert rhythm("[0.9 [0.4 0.8]]") == Rhythm(
        hits=[
            Hit(velocity=0.9, dur=Bar / 2),
            Hit(velocity=0.4, dur=Bar / 4),
            Hit(velocity=0.8, dur=Bar / 4),
        ],
        length_bars=Bar,
    )


def test_rhythm_grammar_generate_plugin_pattern() -> None:
    assert Maj(C | Octave(3)) | cycle([0, 1, 2]) | rhythm("[0.9 [0.4 0.8]]") | name(
        "foo"
    ) == notes("[c3,0.9 [d3,0.4 e3,0.8]]") | name("foo")
