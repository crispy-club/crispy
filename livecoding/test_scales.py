from .scales import (
    Acoustic,
    Maj,
    MajHarm,
    MajHungarian,
    MajLocrian,
    MajNeapolitan,
    MajPent,
    MinHarmonic,
    MinHungarian,
    MinMelodic,
    MinNat,
    MinNeapolitan,
    MinPent,
    cycle,
)
from .pitches import C, Octave


def test_scales_cacoustic() -> None:
    scale = Acoustic(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        52,
        54,
        55,
        57,
        58,
    ]


def test_scales_cmaj() -> None:
    scale = Maj(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        52,
        53,
        55,
        57,
        59,
    ]


def test_scales_cmaj_harm() -> None:
    scale = MajHarm(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        52,
        53,
        55,
        56,
        59,
    ]


def test_scales_cmaj_hungarian() -> None:
    scale = MajHungarian(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        51,
        52,
        54,
        55,
        57,
        58,
    ]


def test_scales_cmaj_locrian() -> None:
    scale = MajLocrian(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        52,
        53,
        54,
        56,
        58,
    ]


def test_scales_cmaj_neapolitan() -> None:
    scale = MajNeapolitan(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        51,
        53,
        55,
        57,
        59,
    ]


def test_scales_cmaj_pent() -> None:
    scale = MajPent(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        52,
        55,
        57,
    ]


def test_scales_cmin_harmonic() -> None:
    scale = MinHarmonic(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        51,
        53,
        55,
        56,
        59,
    ]


def test_scales_cmin_hungarian() -> None:
    scale = MinHungarian(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        51,
        54,
        55,
        56,
        59,
    ]


def test_scales_cmin_melodic() -> None:
    scale = MinMelodic(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        51,
        53,
        55,
        57,
        59,
    ]


def test_scales_cmin_nat() -> None:
    scale = MinNat(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        51,
        53,
        55,
        56,
        58,
    ]


def test_scales_cmin_neapolitan() -> None:
    scale = MinNeapolitan(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        51,
        53,
        55,
        56,
        59,
    ]


def test_scales_cmin_pent() -> None:
    scale = MinPent(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        51,
        53,
        55,
        58,
    ]
