from .scales import (
    Acoustic,
    Altered,
    Augmented,
    Bebop,
    Blues,
    Chromatic,
    Dorian,
    DoubleHarm,
    Enigmatic,
    Flamenco,
    Gypsy,
    HalfDiminished,
    Hirajoshi,
    In,
    Insen,
    Iwato,
    Locrian,
    LocrianSharp6,
    Lydian,
    LydianAugmented,
    LydianDiminished,
    Maj,
    MajHarm,
    MajHungarian,
    MajLocrian,
    MajNeapolitan,
    MajPent,
    MinHarm,
    MinHungarian,
    MinMelodic,
    MinNat,
    MinNeapolitan,
    MinPent,
    Mixolydian,
    Octatonic,
    Persian,
    Phrygian,
    PhrygianDominant,
    Prometheus,
    Tritone,
    TritoneSemi2,
    UkrainianDorian,
    WholeTone,
    Yo,
    cycle,
)
from .pitches import C, Oct, Octave


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


def test_scales_caltered() -> None:
    scale = Altered(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        51,
        52,
        54,
        56,
        58,
    ]


def test_scales_caugmented() -> None:
    scale = Augmented(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        51,
        52,
        55,
        56,
        59,
    ]


def test_scales_cbebop() -> None:
    scale = Bebop(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        52,
        53,
        55,
        57,
        58,
        59,
    ]


def test_scales_cblues() -> None:
    scale = Blues(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        51,
        53,
        54,
        55,
        58,
    ]


def test_scales_cchromatic() -> None:
    scale = Chromatic(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        i + 48 for i in range(12)
    ]


def test_scales_cdorian() -> None:
    scale = Dorian(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        51,
        53,
        55,
        57,
        58,
    ]


def test_scales_cdoubleharm() -> None:
    scale = DoubleHarm(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        52,
        53,
        55,
        56,
        59,
    ]


def test_scales_cenigmatic() -> None:
    scale = Enigmatic(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        52,
        54,
        56,
        58,
        59,
    ]


def test_scales_cflamenco() -> None:
    scale = Flamenco(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        52,
        53,
        55,
        56,
        59,
    ]


def test_scales_cgypsy() -> None:
    scale = Gypsy(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        51,
        54,
        55,
        56,
        58,
    ]


def test_scales_chalfdiminished() -> None:
    scale = HalfDiminished(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        51,
        53,
        54,
        56,
        58,
    ]


def test_scales_chirajoshi() -> None:
    scale = Hirajoshi(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        52,
        54,
        55,
        59,
    ]


def test_scales_cin() -> None:
    scale = In(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        53,
        55,
        56,
    ]


def test_scales_cinsen() -> None:
    scale = Insen(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        53,
        55,
        58,
    ]


def test_scales_ciwato() -> None:
    scale = Iwato(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        53,
        54,
        58,
    ]


def test_scales_clocrian() -> None:
    scale = Locrian(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        51,
        53,
        54,
        56,
        58,
    ]


def test_scales_clocriansharp6() -> None:
    scale = LocrianSharp6(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        51,
        53,
        54,
        57,
        58,
    ]


def test_scales_clydian() -> None:
    scale = Lydian(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        52,
        54,
        55,
        57,
        59,
    ]


def test_scales_clydianaugmented() -> None:
    scale = LydianAugmented(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        52,
        54,
        56,
        57,
        59,
    ]


def test_scales_clydiandiminished() -> None:
    scale = LydianDiminished(C | Oct(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        51,
        54,
        55,
        57,
        59,
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
    scale = MinHarm(C | Octave(2))
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


def test_scales_cmixolydian() -> None:
    scale = Mixolydian(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        52,
        53,
        55,
        57,
        58,
    ]


def test_scales_coctatonic() -> None:
    scale = Octatonic(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        51,
        53,
        54,
        56,
        57,
        59,
    ]


def test_scales_cpersian() -> None:
    scale = Persian(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        52,
        53,
        54,
        56,
        59,
    ]


def test_scales_cphrygian() -> None:
    scale = Phrygian(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        51,
        53,
        55,
        56,
        58,
    ]


def test_scales_cphrygiandominant() -> None:
    scale = PhrygianDominant(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        52,
        53,
        55,
        56,
        58,
    ]


def test_scales_cprometheus() -> None:
    scale = Prometheus(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        52,
        54,
        57,
        58,
    ]


def test_scales_ctritone() -> None:
    scale = Tritone(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        52,
        54,
        55,
        58,
    ]


def test_scales_ctritonesemi2() -> None:
    scale = TritoneSemi2(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        49,
        50,
        54,
        55,
        56,
    ]


def test_scales_cukrainian_dorian() -> None:
    scale = UkrainianDorian(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        51,
        54,
        55,
        57,
        58,
    ]


def test_scales_cwholetone() -> None:
    scale = WholeTone(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        52,
        54,
        56,
        58,
    ]


def test_scales_cyo() -> None:
    scale = Yo(C | Octave(2))
    scale_len = len(scale.pitch_classes())
    assert list(scale | cycle([i for i in range(scale_len)])) == [
        48,
        50,
        53,
        55,
        57,
    ]
