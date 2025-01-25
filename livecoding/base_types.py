import copy
import json
from dataclasses import asdict, dataclass
from math import gcd, lcm
from typing import Callable, Literal, Union, overload

import requests


Rest = Literal["Rest"]
Sharp = Literal["#"]


@dataclass(slots=True)
class Duration:
    num: int
    den: int

    def __init__(self, num: int, den: int) -> None:
        assert den != 0
        self.num = num
        self.den = den

    def json(self) -> str:
        return json.dumps(asdict(self), separators=(",", ":"))

    def __add__(self, other: "Duration") -> "Duration":
        lcmult = lcm(self.den, other.den)
        lnum, rnum = (
            self.num * int(lcmult / self.den),
            other.num * int(lcmult / other.den),
        )
        return Duration(lnum + rnum, lcmult)._simplify()

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Duration):
            return False
        lsimp, rsimp = self._simplify(), other._simplify()
        return lsimp.num == rsimp.num and lsimp.den == rsimp.den

    def __lt__(self, other: "Duration") -> bool:
        lcmult = lcm(self.den, other.den)
        lnum, rnum = (
            self.num * int(lcmult / self.den),
            other.num * int(lcmult / other.den),
        )
        return lnum < rnum

    def __le__(self, other: "Duration") -> bool:
        lcmult = lcm(self.den, other.den)
        lnum, rnum = (
            self.num * int(lcmult / self.den),
            other.num * int(lcmult / other.den),
        )
        return lnum <= rnum

    def __gt__(self, other: "Duration") -> bool:
        lcmult = lcm(self.den, other.den)
        lnum, rnum = (
            self.num * int(lcmult / self.den),
            other.num * int(lcmult / other.den),
        )
        return lnum > rnum

    def __ge__(self, other: "Duration") -> bool:
        lcmult = lcm(self.den, other.den)
        lnum, rnum = (
            self.num * int(lcmult / self.den),
            other.num * int(lcmult / other.den),
        )
        return lnum >= rnum

    @overload
    def __mul__(self, other: "Duration") -> "Duration": ...

    @overload
    def __mul__(self, other: int) -> "Duration": ...

    def __mul__(self, other: Union["Duration", int]) -> "Duration":
        if isinstance(other, int):
            return self.__mul__(Duration(other, 1))
        assert isinstance(other, Duration)
        return Duration(self.num * other.num, self.den * other.den)._simplify()

    def __sub__(self, other: "Duration") -> "Duration":
        return self.__add__(Duration(other.num * -1, other.den))

    @overload
    def __truediv__(self, other: int) -> "Duration": ...

    @overload
    def __truediv__(self, other: "Duration") -> "Duration": ...

    def __truediv__(self, other: Union["Duration", int]) -> "Duration":
        if isinstance(other, int):
            assert other > 0
            return self.__mul__(Duration(1, other))
        assert isinstance(other, Duration)
        return self.__mul__(Duration(other.den, other.num))

    def __rtruediv__(self, n: int) -> "Duration":
        return Duration(num=self.den, den=self.num) * n

    def __repr__(self) -> str:
        return f"({self.num}, {self.den})"

    def __str__(self) -> str:
        return f"{self.num}/{self.den}"

    def _simplify(self) -> "Duration":
        gcdiv = gcd(self.num, self.den)
        if gcdiv == 1:
            return self
        return Duration(int(self.num / gcdiv), int(self.den / gcdiv))


Bar = Duration(1, 1)
Half = Duration(1, 2)
Quarter = Duration(1, 4)
Eighth = Duration(1, 8)
Sixteenth = Duration(1, 16)
Zero = Duration(0, 1)


@dataclass(slots=True)
class Ctrl:
    @dataclass(slots=True)
    class Params:
        cc: int
        value: float

    CtrlEvent: Params

    def __init__(self, ctrl_event: Params) -> None:
        self.CtrlEvent = ctrl_event

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Ctrl):
            return False
        return all(
            [
                getattr(self.CtrlEvent, attr_name)
                == getattr(other.CtrlEvent, attr_name)
                for attr_name in {"cc", "value"}
            ]
        )

    def json(self) -> str:
        return json.dumps(asdict(self), separators=(",", ":"))


@dataclass(slots=True)
class Note:
    @dataclass(slots=True)
    class Params:
        note_num: int
        velocity: float
        dur: Duration

    NoteEvent: Params

    def __init__(self, note_event: Params) -> None:
        self.NoteEvent = note_event

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Note):
            return False
        return all(
            [
                getattr(self.NoteEvent, attr_name)
                == getattr(other.NoteEvent, attr_name)
                for attr_name in {"dur", "note_num", "velocity"}
            ]
        )

    def __mul__(self, mul: Duration) -> "Note":
        return Note(copy.replace(self.NoteEvent, dur=self.NoteEvent.dur * mul))

    def json(self) -> str:
        return json.dumps(asdict(self), separators=(",", ":"))

    def set_dur(self, new_dur: Duration) -> "Note":
        return Note(copy.replace(self.NoteEvent, dur=new_dur))

    def transpose(self, amount: int) -> "Note":
        return Note(
            self.Params(
                note_num=(self.NoteEvent.note_num + amount) % 128,
                velocity=self.NoteEvent.velocity,
                dur=self.NoteEvent.dur,
            ),
        )


@dataclass(slots=True)
class Event:
    action: Ctrl | Note | Rest
    dur: Duration

    def is_ctrl(self) -> bool:
        return isinstance(self.action, Ctrl)

    def is_note(self) -> bool:
        return isinstance(self.action, Note)

    def is_rest(self) -> bool:
        return isinstance(self.action, str) and self.action == "Rest"

    def __mul__(self, mul: Duration) -> "Event":
        if self.is_ctrl() or self.is_rest():
            return copy.replace(self, dur=self.dur * mul)
        # scale the duration of both the note and the containing event
        assert isinstance(self.action, Note)
        self.action.NoteEvent = copy.replace(self.action.NoteEvent, dur=self.dur * mul)
        return copy.replace(self, dur=self.dur * mul)

    def json(self) -> str:
        return json.dumps(asdict(self), separators=(",", ":"))


PluginPatternFilter = Callable[["PluginPattern"], "PluginPattern"]


@dataclass(slots=True)
class PluginPattern:
    name: str
    events: list[Event]
    length_bars: Duration

    def __add__(self, other: "PluginPattern") -> "PluginPattern":
        return PluginPattern(
            name=self.name,
            events=self.events + other.events,
            length_bars=self.length_bars + other.length_bars,
        )

    def __mul__(self, times: int) -> "PluginPattern":
        assert times > 0
        return PluginPattern(
            events=self.events * times,
            length_bars=self.length_bars * times,
            name=self.name,
        )

    def __or__(self, pattern_filter: PluginPatternFilter) -> "PluginPattern":
        return pattern_filter(self)

    def json(self) -> str:
        return json.dumps(asdict(self), separators=(",", ":"))

    def rhythm(self) -> list[Duration]:
        return [ev.dur for ev in self.events]

    def stop(self) -> None:
        resp = requests.post(f"http://127.0.0.1:3000/stop/{self.name}")
        resp.raise_for_status()
