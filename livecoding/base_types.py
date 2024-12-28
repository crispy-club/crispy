import json
from math import gcd, lcm
from typing import Callable, Literal, Union, overload

from attrs import asdict, define


Rest = Literal["Rest"]


@define
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

    def __or__(self, pattern_filter: Callable[["Pattern"], "Pattern"]) -> "Pattern":
        return pattern_filter(self)

    def json(self) -> str:
        return json.dumps(asdict(self), separators=(",", ":"))
