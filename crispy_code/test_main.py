from crispy_code.__main__ import Melody


def test_main_melody_parse() -> None:
    melody = Melody()
    pattern = melody.parse("C3 = [a . .] [o . .] [d .]")
    assert pattern is not None
