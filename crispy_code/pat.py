import copy
import re
from dataclasses import dataclass
from typing import Union

from crispy.base_types import (
    Duration,
    Event,
    Half,
    Note,
    PluginPattern,
)
from crispy.util import random_name


class InvalidSyntaxError(Exception):
    pass


type Octave = int
type PitchClass = int


_DEFAULT_NOTE = 60
_DEFAULT_OCTAVE = 3
_VELOCITY_VALUES = [round((i - 96) / 26, 2) for i in range(97, 123)]
_VELOCITY_TOKENS = {chr(i) for i in range(97, 123)}
_DEFAULT_VELOCITY = _VELOCITY_VALUES[14]
_PITCH_CLASSES: dict[str, PitchClass] = {
    "C": 0,
    "D": 2,
    "E": 4,
    "F": 5,
    "G": 7,
    "A": 9,
    "B": 11,
}


_EMPTY_PATTERN = re.compile(r"^\s*\[\s*\]\s*$")


def _get_note_num(pitch_class: PitchClass, octave: Octave) -> int:
    return pitch_class + ((octave + 2) * 12)


def get_velocity(token: str) -> float:
    return _VELOCITY_VALUES[ord(token[0]) - 97]


@dataclass(slots=True)
class _Alternation:
    children: list[Union[str, "_Alternation", "_Group"]]

    def __repr__(self) -> str:
        return f"<{' '.join([repr(child) for child in self.children])}>"


@dataclass(slots=True)
class _Group:
    children: list[Union[str, "_Alternation", "_Group"]]

    def __repr__(self) -> str:
        return f"[{' '.join([repr(child) for child in self.children])}]"


def _expand_sub_alternations(alt: _Alternation) -> None:
    idx = 0
    new_children: list[str | _Alternation | _Group] = []
    while idx < len(alt.children):
        child = alt.children[idx]
        if not isinstance(child, _Alternation):
            new_children.append(child)
            idx += 1
            continue
        if idx == 0:
            raise InvalidSyntaxError("alternation must have an anchor")
        _expand_sub_alternations(child)
        new_children = new_children[:-1]
        anchor = alt.children[idx - 1]
        for child_a in child.children:
            new_children += [anchor, child_a]
        idx += 1
    alt.children = new_children


def _expand_group_alternations(group: _Group) -> None:
    # Result will be a group with no alternations
    idx = 0
    new_children: list[str | _Alternation | _Group] = []
    while idx < len(group.children):
        child = group.children[idx]
        if not isinstance(child, _Alternation):
            new_children.append(child)
            idx += 1
            continue
        if idx == 0:
            raise InvalidSyntaxError("alternation must have an anchor")
        _expand_sub_alternations(child)
        new_children = new_children[:-1]
        anchor = group.children[idx - 1]
        for child_a in child.children:
            new_children += [anchor, child_a]
        idx += 1
    group.children = new_children


def _parse_note(event_str: str, dur: Duration) -> list[Event]:
    if len(event_str) == 1:
        return [
            Event(
                dur=dur,
                action=Note(
                    Note.Params(
                        note_num=_get_note_num(
                            _PITCH_CLASSES[event_str[0]], _DEFAULT_OCTAVE
                        ),
                        velocity=_DEFAULT_VELOCITY,
                        dur=Half,
                    )
                ),
            ),
        ]
    elif len(event_str) == 2 and event_str[1] == "'":
        return [
            Event(
                dur=dur,
                action=Note(
                    Note.Params(
                        note_num=_get_note_num(
                            _PITCH_CLASSES[event_str[0]] + 1, _DEFAULT_OCTAVE
                        ),
                        velocity=_DEFAULT_VELOCITY,
                        dur=Half,
                    )
                ),
            ),
        ]
    elif len(event_str) == 2 and event_str[1] in _VELOCITY_TOKENS:
        return [
            Event(
                dur=dur,
                action=Note(
                    Note.Params(
                        note_num=_get_note_num(
                            _PITCH_CLASSES[event_str[0]], _DEFAULT_OCTAVE
                        ),
                        velocity=get_velocity(event_str[1]),
                        dur=Half,
                    )
                ),
            ),
        ]
    elif len(event_str) == 2:
        # second char is an int that specifies an octave
        return [
            Event(
                dur=dur,
                action=Note(
                    Note.Params(
                        note_num=_get_note_num(
                            _PITCH_CLASSES[event_str[0]], int(event_str[1])
                        ),
                        velocity=_DEFAULT_VELOCITY,
                        dur=Half,
                    )
                ),
            ),
        ]
    elif len(event_str) == 3 and event_str[1] == "'":
        # examples: C'1 C'x
        if event_str[2] in _VELOCITY_TOKENS:
            return [
                Event(
                    dur=dur,
                    action=Note(
                        Note.Params(
                            note_num=_get_note_num(
                                _PITCH_CLASSES[event_str[0]] + 1, _DEFAULT_OCTAVE
                            ),
                            velocity=get_velocity(event_str[2]),
                            dur=Half,
                        )
                    ),
                ),
            ]
        else:
            return [
                Event(
                    dur=dur,
                    action=Note(
                        Note.Params(
                            note_num=_get_note_num(
                                _PITCH_CLASSES[event_str[0]] + 1, int(event_str[2])
                            ),
                            velocity=_DEFAULT_VELOCITY,
                            dur=Half,
                        )
                    ),
                ),
            ]
    elif len(event_str) == 3:
        # example: C1x
        return [
            Event(
                dur=dur,
                action=Note(
                    Note.Params(
                        note_num=_get_note_num(
                            _PITCH_CLASSES[event_str[0]], int(event_str[1])
                        ),
                        velocity=get_velocity(event_str[2]),
                        dur=Half,
                    )
                ),
            ),
        ]
    if len(event_str) != 4:
        raise InvalidSyntaxError()
    if event_str[1] != "'":
        raise InvalidSyntaxError(f"expected {event_str[1]} to equal ' (single quote)")
    return [
        Event(
            dur=dur,
            action=Note(
                Note.Params(
                    note_num=_get_note_num(
                        _PITCH_CLASSES[event_str[0]] + 1, int(event_str[2])
                    ),
                    velocity=get_velocity(event_str[3]),
                    dur=Half,
                )
            ),
        ),
    ]


def _parse(event_str: str, dur: Duration) -> list[Event]:
    event_str = event_str.strip()
    if len(event_str) == 0:
        return []
    for note in _PITCH_CLASSES:
        if event_str.startswith(note):
            return _parse_note(event_str, dur)
    if event_str == ".":
        return [
            Event(action="Rest", dur=dur),
        ]
    assert event_str[0] in _VELOCITY_TOKENS, event_str
    return [
        Event(
            action=Note(
                Note.Params(
                    note_num=_DEFAULT_NOTE,
                    velocity=get_velocity(event_str[0]),
                    dur=Half,
                )
            ),
            dur=dur,
        ),
    ]


def _transform(groups_tree: _Group, length_bars: Duration) -> list[Event]:
    if len(groups_tree.children) == 0:
        return []
    each_dur = length_bars / len(groups_tree.children)
    events: list[Event] = []
    for child in groups_tree.children:
        if isinstance(child, _Group):
            inner_events = _transform(child, each_dur)
            events += inner_events
            continue
        assert isinstance(child, str)
        if child == "_":
            if len(events) == 0:
                raise InvalidSyntaxError()
            events = events[:-1] + [
                copy.replace(events[-1], dur=events[-1].dur + each_dur)
            ]
        else:
            events += _parse(child, each_dur)
    return events


def _remove_leading(tokens: list[str], undesired: set[str]) -> list[str]:
    for idx, char in enumerate(tokens):
        if char in undesired:
            continue
        return tokens[idx + 1 :]
    return tokens


def _get_subgroups_r(
    group: Union[_Alternation, _Group], tokens: list[str]
) -> list[str]:
    if len(tokens) == 0:
        return []
    tok = tokens[0]
    remainder: list[str] = []
    if tok == "[":
        child_g = _Group(children=[])
        remainder = _get_subgroups_r(child_g, tokens[1:])
        group.children.append(child_g)
        return _get_subgroups_r(group, remainder)
    elif tok == "]":
        return tokens[1:]
    elif tok == "<":
        child_a = _Alternation(children=[])
        remainder = _get_subgroups_r(child_a, tokens[1:])
        group.children.append(child_a)
        return _get_subgroups_r(group, remainder)
    elif tok == ">":
        return tokens[1:]
    elif _EMPTY_PATTERN.match(tok):
        _get_subgroups_r(group, tokens[1:])
    else:
        group.children.append(tok)
        return _get_subgroups_r(group, tokens[1:])
    return []


def _get_groups(definition: str) -> _Group:
    root = _Group(children=[])
    if _EMPTY_PATTERN.match(definition) is not None:
        return root
    definition = definition.strip()
    if (definition[0] != "[") or (definition[-1] != "]"):
        raise InvalidSyntaxError()
    try:
        sepd = _separate_delimiters(definition[1:-1].split())
        _get_subgroups_r(root, sepd)
    except InvalidSyntaxError:
        raise
    _expand_group_alternations(root)
    return root


def _expand(atom: str) -> list[str]:
    if ";" in atom:
        pieces = atom.split(";")
        if len(pieces) != 2:
            raise InvalidSyntaxError()
        try:
            n = int(pieces[1])
        except ValueError:
            raise InvalidSyntaxError()
        assert n > 1, atom
        return ["["] + ([pieces[0]] * n) + ["]"]
    elif ":" in atom:
        pieces = atom.split(":")
        if len(pieces) != 2:
            raise InvalidSyntaxError()
        try:
            n = int(pieces[1])
        except ValueError:
            raise InvalidSyntaxError()
        assert n > 1, atom
        return [pieces[0]] * n
    elif "@" in atom:
        pieces = atom.split("@")
        if len(pieces) != 2:
            raise InvalidSyntaxError()
        try:
            n = int(pieces[1])
        except ValueError:
            raise InvalidSyntaxError()
        assert n > 1, atom
        return [pieces[0]] + (["_"] * (n - 1))
    return [atom]


def _separate_delimiters(tokens: list[str]) -> list[str]:
    if len(tokens) == 0:
        return []
    tok = tokens[0].strip()
    assert len(tok) > 0
    if len(tok) == 1:
        return _expand(tok) + _separate_delimiters(tokens[1:])
    if tok[0] == "[":
        return ["["] + _separate_delimiters([tok[1:]] + tokens[1:])
    if tok[-1] == "]":
        return _separate_delimiters([tok[:-1], "]"] + tokens[1:])
    if tok[0] == "<":
        return ["<"] + _separate_delimiters([tok[1:]] + tokens[1:])
    if tok[-1] == ">":
        return _separate_delimiters([tok[:-1], ">"] + tokens[1:])
    return _expand(tok) + _separate_delimiters(tokens[1:])


def _parse_pattern(
    definition: str,
    length_bars: Duration = Duration(1, 1),
) -> list[Event]:
    groups_tree = _get_groups(definition)
    return _transform(groups_tree, length_bars)


def pat(
    definition: str,
    length_bars: Duration = Duration(1, 1),
) -> PluginPattern:
    return PluginPattern(
        events=_parse_pattern(definition, length_bars),
        length_bars=length_bars,
        name=random_name(),
    )
