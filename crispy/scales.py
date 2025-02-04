from abc import ABC
from collections.abc import Iterator
from dataclasses import dataclass
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


@dataclass(slots=True)
class cycle:
    index: list[int]

    def __call__(self, scale: Scale | Iterator[Scale]) -> Iterator[int]:
        if isinstance(scale, Scale):
            return iter([scale[i] for i in self.index])
        return iter([scl[i] for scl in scale for i in self.index])


class Acoustic(Scale):
    _pitchclasses = (0, 2, 4, 6, 7, 9, 10)


class Altered(Scale):
    _pitchclasses = (0, 1, 3, 4, 6, 8, 10)


class Augmented(Scale):
    _pitchclasses = (0, 3, 4, 7, 8, 11)


class Bebop(Scale):
    _pitchclasses = (0, 2, 4, 5, 7, 9, 10, 11)


class Blues(Scale):
    _pitchclasses = (0, 3, 5, 6, 7, 10)


class Chromatic(Scale):
    _pitchclasses = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)


class Dorian(Scale):
    _pitchclasses = (0, 2, 3, 5, 7, 9, 10)


class DoubleHarm(Scale):
    _pitchclasses = (0, 1, 4, 5, 7, 8, 11)


class Enigmatic(Scale):
    _pitchclasses = (0, 1, 4, 6, 8, 10, 11)


class Flamenco(Scale):
    _pitchclasses = (0, 1, 4, 5, 7, 8, 11)


class Gypsy(Scale):
    _pitchclasses = (0, 2, 3, 6, 7, 8, 10)


class HalfDiminished(Scale):
    _pitchclasses = (0, 2, 3, 5, 6, 8, 10)


class Hirajoshi(Scale):
    _pitchclasses = (0, 4, 6, 7, 11)


class In(Scale):
    _pitchclasses = (0, 1, 5, 7, 8)


class Insen(Scale):
    _pitchclasses = (0, 1, 5, 7, 10)


class Iwato(Scale):
    _pitchclasses = (0, 1, 5, 6, 10)


class Locrian(Scale):
    _pitchclasses = (0, 1, 3, 5, 6, 8, 10)


class LocrianSharp6(Scale):
    _pitchclasses = (0, 1, 3, 5, 6, 9, 10)


class Lydian(Scale):
    _pitchclasses = (0, 2, 4, 6, 7, 9, 11)


class LydianAugmented(Scale):
    _pitchclasses = (0, 2, 4, 6, 8, 9, 11)


class LydianDiminished(Scale):
    _pitchclasses = (0, 2, 3, 6, 7, 9, 11)


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


class MinHarm(Scale):
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


class Mixolydian(Scale):
    _pitchclasses = (0, 2, 4, 5, 7, 9, 10)


class Octatonic(Scale):
    _pitchclasses = (0, 2, 3, 5, 6, 8, 9, 11)


class Persian(Scale):
    _pitchclasses = (0, 1, 4, 5, 6, 8, 11)


class Phrygian(Scale):
    _pitchclasses = (0, 1, 3, 5, 7, 8, 10)


class PhrygianDominant(Scale):
    _pitchclasses = (0, 1, 4, 5, 7, 8, 10)


class Prometheus(Scale):
    _pitchclasses = (0, 2, 4, 6, 9, 10)


class Tritone(Scale):
    _pitchclasses = (0, 1, 4, 6, 7, 10)


class TritoneSemi2(Scale):
    _pitchclasses = (0, 1, 2, 6, 7, 8)


class UkrainianDorian(Scale):
    _pitchclasses = (0, 2, 3, 6, 7, 9, 10)


class WholeTone(Scale):
    _pitchclasses = (0, 2, 4, 6, 8, 10)


class Yo(Scale):
    _pitchclasses = (0, 2, 5, 7, 9)


Ionian = Maj
