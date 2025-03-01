import json
from unittest import mock

import pytest
from requests.exceptions import HTTPError

from crispy.base_types import Duration, Event, Note, PluginPattern
from crispy.filters import name
from crispy.pat import pat
from crispy.plugin import ch2, play


def test_crispy_duration_addition() -> None:
    sixteenth = Duration(1, 16)
    assert sixteenth + sixteenth == Duration(1, 8)


def test_crispy_duration_mult() -> None:
    assert Duration(1, 16) * Duration(2, 3) == Duration(1, 24)


def test_crispy_duration_division() -> None:
    assert Duration(1, 16) / Duration(2, 3) == Duration(3, 32)


def test_pat_simple() -> None:
    pattern = pat("[C1 D'1 G1 C2]") | name("bassline")
    print(pattern)
    assert pattern == PluginPattern(
        name="bassline",
        length_bars=Duration(1, 1),
        events=[
            Event(
                action=Note(
                    Note.Params(note_num=36, velocity=0.58, dur=Duration(1, 2))
                ),
                dur=Duration(1, 4),
            ),
            Event(
                action=Note(
                    Note.Params(note_num=39, velocity=0.58, dur=Duration(1, 2))
                ),
                dur=Duration(1, 4),
            ),
            Event(
                action=Note(
                    Note.Params(note_num=43, velocity=0.58, dur=Duration(1, 2))
                ),
                dur=Duration(1, 4),
            ),
            Event(
                action=Note(
                    Note.Params(note_num=48, velocity=0.58, dur=Duration(1, 2))
                ),
                dur=Duration(1, 4),
            ),
        ],
    )


def test_pat_nested() -> None:
    pattern = pat("[C1 [D'1 C1 D'1] G1 C2]") | name("bassline")
    print(pattern)
    assert pattern == PluginPattern(
        name="bassline",
        length_bars=Duration(1, 1),
        events=[
            Event(
                action=Note(
                    Note.Params(note_num=36, velocity=0.58, dur=Duration(1, 2))
                ),
                dur=Duration(1, 4),
            ),
            Event(
                action=Note(
                    Note.Params(note_num=39, velocity=0.58, dur=Duration(1, 2))
                ),
                dur=Duration(1, 12),
            ),
            Event(
                action=Note(
                    Note.Params(note_num=36, velocity=0.58, dur=Duration(1, 2))
                ),
                dur=Duration(1, 12),
            ),
            Event(
                action=Note(
                    Note.Params(note_num=39, velocity=0.58, dur=Duration(1, 2))
                ),
                dur=Duration(1, 12),
            ),
            Event(
                action=Note(
                    Note.Params(note_num=43, velocity=0.58, dur=Duration(1, 2))
                ),
                dur=Duration(1, 4),
            ),
            Event(
                action=Note(
                    Note.Params(note_num=48, velocity=0.58, dur=Duration(1, 2))
                ),
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
        play(pat("[C1 [D'1 C1 D'1] G1 C2]") | name("bassline"))

    assert str(herr.value) == "invalid pattern"


@mock.patch("requests.post")
def test_plugin_play_note_pattern_on_channel2(mock_post: mock.Mock) -> None:
    mock_resp = mock.Mock()
    mock_resp.status_code = 201
    mock_post.return_value = mock_resp
    ch2 << (pat("[C1 G1]") | name("bassline"))
    mock_post.assert_called_with(
        "http://127.0.0.1:3000/start/ch2",
        headers={"Content-Type": "application/json"},
        data=json.dumps(
            {
                "events": [
                    {
                        "action": {
                            "NoteEvent": {
                                "note_num": 36,
                                "velocity": 0.58,
                                "dur": {"num": 1, "den": 2},
                            },
                        },
                        "dur": {"num": 1, "den": 2},
                    },
                    {
                        "action": {
                            "NoteEvent": {
                                "note_num": 43,
                                "velocity": 0.58,
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
