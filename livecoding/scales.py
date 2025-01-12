from attrs import define

from abc import ABC
from collections.abc import Iterator
from typing import Protocol


class ScaleIndexer(Protocol):
    def __call__(self, scale: "Scale") -> Iterator[int]: ...


class Scale(ABC):
    _pitchclasses: tuple[int, ...]
    _tonic: int

    def __init__(self, tonic: int) -> None:
        self._tonic = tonic

    def __getitem__(self, index: int) -> int:
        pcs = self.pitch_classes()
        num_pcs = len(pcs)
        index = index % num_pcs
        return self.tonic() + pcs[index]

    def __or__(self, indexer: ScaleIndexer) -> Iterator[int]:
        return indexer(self)

    def tonic(self) -> int:
        return self._tonic

    def pitch_classes(self) -> tuple[int, ...]:
        return self._pitchclasses


@define
class cycle:
    index: list[int]

    def __call__(self, scale: Scale | Iterator[Scale]) -> Iterator[int]:
        if isinstance(scale, Scale):
            return iter([scale[i] for i in self.index])
        return iter([scl[i] for scl in scale for i in self.index])


class Acoustic(Scale):
    _pitchclasses = (0, 2, 4, 6, 7, 9, 10)


class Maj(Scale):
    _pitchclasses = (0, 2, 4, 5, 7, 9, 11)


class MajHarm(Scale):
    _pitchclasses = (0, 2, 4, 5, 7, 8, 11)


class MajHungarian(Scale):
    _pitchclasses = (0, 3, 4, 6, 7, 9, 10)


class MajLocrian(Scale):
    _pitchclasses = (0, 2, 4, 5, 6, 8, 10)


class MajNeapolitan(Scale):
    _pitchclasses = (0, 1, 3, 5, 7, 9, 11)


class MajPent(Scale):
    _pitchclasses = (0, 2, 4, 7, 9)


class MinHarmonic(Scale):
    _pitchclasses = (0, 2, 3, 5, 7, 8, 11)


class MinHungarian(Scale):
    _pitchclasses = (0, 2, 3, 6, 7, 8, 11)


class MinMelodic(Scale):
    _pitchclasses = (0, 2, 3, 5, 7, 9, 11)


class MinNat(Scale):
    _pitchclasses = (0, 2, 3, 5, 7, 8, 10)


class MinNeapolitan(Scale):
    _pitchclasses = (0, 1, 3, 5, 7, 8, 11)


class MinPent(Scale):
    _pitchclasses = (0, 3, 5, 7, 10)


Ionian = Maj
