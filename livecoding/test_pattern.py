from livecoding.base_types import Duration, Event, Note, Pattern
from livecoding.pattern import rev, transpose


def test_rev() -> None:
    foo = Pattern(
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
            Event(
                action=Note(
                    Note.Params(
                        note_num=63,
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
    assert rev(foo) == Pattern(
        events=[
            Event(
                action=Note(
                    Note.Params(
                        note_num=63,
                        velocity=0.9,
                        dur_ms=20,
                    ),
                ),
                dur_frac=Duration(1, 4),
            ),
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


def test_transpose() -> None:
    foo = Pattern(
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
            Event(
                action=Note(
                    Note.Params(
                        note_num=63,
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
    assert transpose(7, foo) == Pattern(
        events=[
            Event(
                action=Note(
                    Note.Params(
                        note_num=67,
                        velocity=0.9,
                        dur_ms=20,
                    ),
                ),
                dur_frac=Duration(1, 4),
            ),
            Event(
                action=Note(
                    Note.Params(
                        note_num=70,
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
