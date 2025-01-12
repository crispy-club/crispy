from abc import ABC

from attrs import define


@define
class PitchClass(ABC):
    _offset: int

    def __int__(self) -> int:
        return self._offset

    def sharp(self) -> "PitchClass":
        return PitchClass(self._offset + 1)


C = PitchClass(24)
D = PitchClass(26)
E = PitchClass(28)
F = PitchClass(29)
G = PitchClass(31)
A = PitchClass(33)
B = PitchClass(35)


@define
class Octave:
    num: int

    def __ror__(self, pitch_class: PitchClass) -> int:
        return (self.num * 12) + int(pitch_class)
