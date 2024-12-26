from livecoding.base_types import Duration, Event, Note, Pattern


def test_duration_division() -> None:
    one_bar = Duration(1, 1)
    assert one_bar / 4 == Duration(1, 4)


def test_note_eq() -> None:
    n1 = Note(Note.Params(36, 0.8, 20))
    n2 = Note(Note.Params(36, 0.8, 20))
    assert n1 == n2


def test_duration_json() -> None:
    dur = Duration(1, 1)
    assert dur.json() == """{"num":1,"den":1}"""


def test_note_json() -> None:
    note = Note(
        Note.Params(
            note_num=60,
            velocity=0.9,
            dur_ms=20,
        ),
    )
    assert note.json() == """{"NoteEvent":{"note_num":60,"velocity":0.9,"dur_ms":20}}"""


def test_pattern_json() -> None:
    pattern = Pattern(
        events=[
            Event(
                action=Note(
                    Note.Params(
                        note_num=60,
                        velocity=0.9,
                        dur_ms=20,
                    ),
                ),
                dur_frac=Duration(1, 4),
            ),
        ],
        length_bars=Duration(1, 1),
        name="foo",
    )
    assert (
        pattern.json()
        == """{"events":[{"action":{"NoteEvent":{"note_num":60,"velocity":0.9,"dur_ms":20}},"dur_frac":{"num":1,"den":4}}],"length_bars":{"num":1,"den":1},"name":"foo"}"""
    )
