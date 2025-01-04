import json
from unittest import mock

import pytest
from requests.exceptions import HTTPError

from livecoding.base_types import Duration, Event, Note, NotePattern
from livecoding.grammar import notes
from livecoding.pattern import name
from livecoding.plugin import ch2, play


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


@mock.patch("requests.post")
def test_plugin_play_note_pattern_error(mock_post: mock.Mock) -> None:
    mock_resp = mock.Mock()
    mock_resp.status_code = 422
    mock_resp.raise_for_status = mock.Mock()
    mock_resp.raise_for_status.side_effect = HTTPError("invalid pattern")
    mock_post.return_value = mock_resp

    with pytest.raises(HTTPError) as herr:
        play(notes("[c1 [d#1 c1 d#1] g1 c2]") | name("bassline"))

    assert str(herr.value) == "invalid pattern"


@mock.patch("requests.post")
def test_plugin_play_note_pattern_on_channel2(mock_post: mock.Mock) -> None:
    mock_resp = mock.Mock()
    mock_resp.status_code = 201
    mock_post.return_value = mock_resp
    ch2 << (notes("[c1 g1]") | name("bassline"))
    mock_post.assert_called_with(
        "http://127.0.0.1:3000/start/bassline",
        headers={"Content-Type": "application/json"},
        data=json.dumps(
            {
                "events": [
                    {
                        "action": {
                            "NoteEvent": {
                                "note_num": 36,
                                "velocity": 0.8,
                                "dur": {"num": 1, "den": 2},
                            },
                        },
                        "dur": {"num": 1, "den": 2},
                    },
                    {
                        "action": {
                            "NoteEvent": {
                                "note_num": 43,
                                "velocity": 0.8,
                                "dur": {"num": 1, "den": 2},
                            },
                        },
                        "dur": {"num": 1, "den": 2},
                    },
                ],
                "channel": 2,
            }
        ),
    )
