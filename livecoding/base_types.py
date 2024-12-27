import json
from math import gcd, lcm
from typing import Literal, Union, overload

from attrs import asdict, define


Rest = Literal["Rest"]


@define
class Duration:
    num: int
    den: int

    def __init__(self, num: int, den: int) -> None:
        assert num > 0 and den > 0
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
        gcdiv = gcd(lnum, rnum)
        if gcdiv == 1:
            return Duration(lnum + rnum, lcmult)._simplify()
        return Duration(int(lnum / gcdiv) + int(rnum / gcdiv), lcmult)

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Duration):
            return False
        lsimp, rsimp = self._simplify(), other._simplify()
        return lsimp.num == rsimp.num and lsimp.den == rsimp.den

    @overload
    def __mul__(self, other: "Duration") -> "Duration": ...

    @overload
    def __mul__(self, other: int) -> "Duration": ...

    def __mul__(self, other: Union["Duration", int]) -> "Duration":
        if isinstance(other, int):
            return self.__mul__(Duration(other, 1))
        assert isinstance(other, Duration)
        return Duration(self.num * other.num, self.den * other.den)._simplify()

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

    def __repr__(self) -> str:
        return f"({self.num}, {self.den})"

    def __str__(self) -> str:
        return f"({self.num}, {self.den})"

    def _simplify(self) -> "Duration":
        gcdiv = gcd(self.num, self.den)
        if gcdiv == 1:
            return self
        return Duration(int(self.num / gcdiv), int(self.den / gcdiv))


@define
class Note:
    @define
    class Params:
        note_num: int
        velocity: float
        dur_ms: int

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
                for attr_name in {"dur_ms", "note_num", "velocity"}
            ]
        )

    def json(self) -> str:
        return json.dumps(asdict(self), separators=(",", ":"))

    def transpose(self, amount: int) -> "Note":
        return Note(
            self.Params(
                note_num=(self.NoteEvent.note_num + amount) % 128,
                velocity=self.NoteEvent.velocity,
                dur_ms=self.NoteEvent.dur_ms,
            ),
        )


@define
class Event:
    action: Rest | Note
    dur_frac: Duration

    def json(self) -> str:
        return json.dumps(asdict(self), separators=(",", ":"))


@define
class Pattern:
    events: list[Event]
    length_bars: Duration
    name: str

    def __add__(self, other: "Pattern") -> "Pattern":
        return Pattern(
            events=self.events + other.events,
            length_bars=self.length_bars + other.length_bars,
            name=self.name,
        )

    def __mul__(self, times: int) -> "Pattern":
        assert times > 0
        return Pattern(
            events=self.events * times,
            length_bars=self.length_bars * times,
            name=self.name,
        )

    def json(self) -> str:
        return json.dumps(asdict(self), separators=(",", ":"))
