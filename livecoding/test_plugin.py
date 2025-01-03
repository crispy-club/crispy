from livecoding.base_types import Duration, Event, Note, NotePattern
from livecoding.grammar import notes
from livecoding.pattern import name


def test_livecoding_duration_addition() -> None:
    sixteenth = Duration(1, 16)
    assert sixteenth + sixteenth == Duration(1, 8)


def test_livecoding_duration_mult() -> None:
    assert Duration(1, 16) * Duration(2, 3) == Duration(1, 24)


def test_livecoding_duration_division() -> None:
    assert Duration(1, 16) / Duration(2, 3) == Duration(3, 32)


def test_notes_simple() -> None:
    pattern = notes("[c1 d#1 g1 c2]") | name("bassline")
    print(pattern)
    assert pattern == NotePattern(
        name="bassline",
        length_bars=Duration(1, 1),
        events=[
            Event(
                action=Note(Note.Params(note_num=36, velocity=0.8, dur=Duration(1, 2))),
                dur=Duration(1, 4),
            ),
            Event(
                action=Note(Note.Params(note_num=39, velocity=0.8, dur=Duration(1, 2))),
                dur=Duration(1, 4),
            ),
            Event(
                action=Note(Note.Params(note_num=43, velocity=0.8, dur=Duration(1, 2))),
                dur=Duration(1, 4),
            ),
            Event(
                action=Note(Note.Params(note_num=48, velocity=0.8, dur=Duration(1, 2))),
                dur=Duration(1, 4),
            ),
        ],
    )


def test_notes_nested() -> None:
    pattern = notes("[c1 [d#1 c1 d#1] g1 c2]") | name("bassline")
    print(pattern)
    assert pattern == NotePattern(
        name="bassline",
        length_bars=Duration(1, 1),
        events=[
            Event(
                action=Note(Note.Params(note_num=36, velocity=0.8, dur=Duration(1, 2))),
                dur=Duration(1, 4),
            ),
            Event(
                action=Note(Note.Params(note_num=39, velocity=0.8, dur=Duration(1, 2))),
                dur=Duration(1, 12),
            ),
            Event(
                action=Note(Note.Params(note_num=36, velocity=0.8, dur=Duration(1, 2))),
                dur=Duration(1, 12),
            ),
            Event(
                action=Note(Note.Params(note_num=39, velocity=0.8, dur=Duration(1, 2))),
                dur=Duration(1, 12),
            ),
            Event(
                action=Note(Note.Params(note_num=43, velocity=0.8, dur=Duration(1, 2))),
                dur=Duration(1, 4),
            ),
            Event(
                action=Note(Note.Params(note_num=48, velocity=0.8, dur=Duration(1, 2))),
                dur=Duration(1, 4),
            ),
        ],
    )
