from livecoding.base_types import Duration, Note


def test_duration_division() -> None:
    one_bar = Duration(1, 1)
    assert one_bar / 4 == Duration(1, 4)


def test_note_eq() -> None:
    n1 = Note(36, 0.8, 20)
    n2 = Note(36, 0.8, 20)
    assert n1 == n2
