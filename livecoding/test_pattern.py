from livecoding.base_types import Bar, Event, Note, Pattern
from livecoding.pattern import rev, rot, tran
from livecoding.plugin import note_pattern


def test_empty_pattern() -> None:
    assert note_pattern("foo", "[]") == Pattern(
        name="foo",
        length_bars=Bar,
        events=[],
    )


def test_pattern_with_velocity() -> None:
    assert note_pattern("foo", "[c2,0.8 g2,0.7]") == Pattern(
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
    print(f">>>>>>>>>>>>>>>>>>> {res}")
    assert res == Pattern(
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
