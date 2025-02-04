from crispy.pitches import C, D, Octave
from crispy.scales import Maj


def test_scale_index() -> None:
    scale = Maj(C | Octave(3))
    assert scale[2] == 64

    scale = Maj(D.sharp() | Octave(3))
    assert scale[2] == 67
