from livecoding.base_types import Duration


def test_livecoding_duration_addition() -> None:
    sixteenth = Duration(1, 16)
    assert sixteenth + sixteenth == Duration(1, 8)


def test_livecoding_duration_mult() -> None:
    assert Duration(1, 16) * Duration(2, 3) == Duration(1, 24)


def test_livecoding_duration_division() -> None:
    assert Duration(1, 16) / Duration(2, 3) == Duration(3, 32)
