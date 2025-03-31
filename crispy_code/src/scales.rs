#[allow(dead_code)]
struct Scale {
    pitchclasses: Vec<u8>,
}

#[allow(dead_code)]
impl Scale {
    fn acoustic() -> Self {
        Self {
            pitchclasses: vec![0, 2, 4, 6, 7, 9, 10],
        }
    }

    fn altered() -> Self {
        Self {
            pitchclasses: vec![0, 1, 3, 4, 6, 8, 10],
        }
    }

    fn augmented() -> Self {
        Self {
            pitchclasses: vec![0, 3, 4, 7, 8, 11],
        }
    }

    fn bebop() -> Self {
        Self {
            pitchclasses: vec![0, 2, 4, 5, 7, 9, 10, 11],
        }
    }

    fn blues() -> Self {
        Self {
            pitchclasses: vec![0, 3, 5, 6, 7, 10],
        }
    }

    fn chromatic() -> Self {
        Self {
            pitchclasses: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        }
    }

    fn dorian() -> Self {
        Self {
            pitchclasses: vec![0, 2, 3, 5, 7, 9, 10],
        }
    }

    fn double_harm() -> Self {
        Self {
            pitchclasses: vec![0, 1, 4, 5, 7, 8, 11],
        }
    }

    fn enigmatic() -> Self {
        Self {
            pitchclasses: vec![0, 1, 4, 6, 8, 10, 11],
        }
    }

    fn flamenco() -> Self {
        Self {
            pitchclasses: vec![0, 1, 4, 5, 7, 8, 11],
        }
    }

    fn gypsy() -> Self {
        Self {
            pitchclasses: vec![0, 2, 3, 6, 7, 8, 10],
        }
    }

    fn half_diminished() -> Self {
        Self {
            pitchclasses: vec![0, 2, 3, 5, 6, 8, 10],
        }
    }

    fn hirajoshi() -> Self {
        Self {
            pitchclasses: vec![0, 4, 6, 7, 11],
        }
    }

    fn insen() -> Self {
        Self {
            pitchclasses: vec![0, 1, 5, 7, 10],
        }
    }

    fn ionian() -> Self {
        Self::maj()
    }

    fn iwato() -> Self {
        Self {
            pitchclasses: vec![0, 1, 5, 6, 10],
        }
    }

    fn locrian() -> Self {
        Self {
            pitchclasses: vec![0, 1, 3, 5, 6, 8, 10],
        }
    }

    fn locrian_sharp6() -> Self {
        Self {
            pitchclasses: vec![0, 1, 3, 5, 6, 9, 10],
        }
    }

    fn lydian() -> Self {
        Self {
            pitchclasses: vec![0, 2, 4, 6, 7, 9, 11],
        }
    }

    fn lydian_augmented() -> Self {
        Self {
            pitchclasses: vec![0, 2, 4, 6, 8, 9, 11],
        }
    }

    fn lydian_diminished() -> Self {
        Self {
            pitchclasses: vec![0, 2, 3, 6, 7, 9, 11],
        }
    }

    fn maj() -> Self {
        Self {
            pitchclasses: vec![0, 2, 4, 5, 7, 9, 11],
        }
    }

    fn maj_harm() -> Self {
        Self {
            pitchclasses: vec![0, 2, 4, 5, 7, 8, 11],
        }
    }

    fn maj_hungarian() -> Self {
        Self {
            pitchclasses: vec![0, 3, 4, 6, 7, 9, 10],
        }
    }

    fn maj_locrian() -> Self {
        Self {
            pitchclasses: vec![0, 2, 4, 5, 6, 8, 10],
        }
    }

    fn maj_neapolitan() -> Self {
        Self {
            pitchclasses: vec![0, 1, 3, 5, 7, 9, 11],
        }
    }

    fn maj_pent() -> Self {
        Self {
            pitchclasses: vec![0, 2, 4, 7, 9],
        }
    }

    fn min_harm() -> Self {
        Self {
            pitchclasses: vec![0, 2, 3, 5, 7, 8, 11],
        }
    }

    fn min_hungarian() -> Self {
        Self {
            pitchclasses: vec![0, 2, 3, 6, 7, 8, 11],
        }
    }

    fn min_melodic() -> Self {
        Self {
            pitchclasses: vec![0, 2, 3, 5, 7, 9, 11],
        }
    }

    fn min_nat() -> Self {
        Self {
            pitchclasses: vec![0, 2, 3, 5, 7, 8, 10],
        }
    }

    fn min_neapolitan() -> Self {
        Self {
            pitchclasses: vec![0, 1, 3, 5, 7, 8, 11],
        }
    }

    fn min_pent() -> Self {
        Self {
            pitchclasses: vec![0, 3, 5, 7, 10],
        }
    }

    fn mixolydian() -> Self {
        Self {
            pitchclasses: vec![0, 2, 4, 5, 7, 9, 10],
        }
    }

    fn octatonic() -> Self {
        Self {
            pitchclasses: vec![0, 2, 3, 5, 6, 8, 9, 11],
        }
    }

    fn persian() -> Self {
        Self {
            pitchclasses: vec![0, 1, 4, 5, 6, 8, 11],
        }
    }

    fn phrygian() -> Self {
        Self {
            pitchclasses: vec![0, 1, 3, 5, 7, 8, 10],
        }
    }

    fn phrygian_dominant() -> Self {
        Self {
            pitchclasses: vec![0, 1, 4, 5, 7, 8, 10],
        }
    }

    fn prometheus() -> Self {
        Self {
            pitchclasses: vec![0, 2, 4, 6, 9, 10],
        }
    }

    fn tritone() -> Self {
        Self {
            pitchclasses: vec![0, 1, 4, 6, 7, 10],
        }
    }

    fn tritone_semi2() -> Self {
        Self {
            pitchclasses: vec![0, 1, 2, 6, 7, 8],
        }
    }

    fn ukrainian_dorian() -> Self {
        Self {
            pitchclasses: vec![0, 2, 3, 6, 7, 9, 10],
        }
    }

    fn whole_tone() -> Self {
        Self {
            pitchclasses: vec![0, 2, 4, 6, 8, 10],
        }
    }

    fn yo() -> Self {
        Self {
            pitchclasses: vec![0, 2, 5, 7, 9],
        }
    }
}
