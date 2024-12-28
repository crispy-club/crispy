from livecoding.pattern import rev, rot, tran
from livecoding.plugin import note_pattern


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
