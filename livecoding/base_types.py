from math import gcd, lcm
from typing import Literal, NamedTuple


Rest = Literal["Rest"]


class Duration:
    num: int
    den: int

    def __init__(self, num: int, den: int) -> None:
        assert num > 0 and den > 0
        self.num = num
        self.den = den

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

    def __mul__(self, other: "Duration") -> "Duration":
        return Duration(self.num * other.num, self.den * other.den)

    def __truediv__(self, other: "Duration") -> "Duration":
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


class Note:
    class InnerEvent(NamedTuple):
        note_num: int
        velocity: float
        dur_ms: int

    NoteEvent: InnerEvent

    def __init__(self, note_num: int, velocity: float, dur_ms: int) -> None:
        self.NoteEvent = self.InnerEvent(
            note_num=note_num,
            velocity=velocity,
            dur_ms=dur_ms,
        )


class Event(NamedTuple):
    action: Rest | Note
    dur_frac: Duration


class Pattern(NamedTuple):
    events: list[Event]
    length_bars: Duration
    name: str
