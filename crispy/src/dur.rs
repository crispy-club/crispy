use num::integer::gcd;
use num::integer::lcm;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};

pub static BAR: Dur = Dur { num: 1, den: 1 };
// static HALF: Dur = Dur::new(1, 2);
// static QUARTER: Dur = Dur::new(1, 4);
// static EIGHTH: Dur = Dur::new(1, 8);
// static SIXTEENTH: Dur = Dur::new(1, 16);

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Dur {
    pub num: i64,
    pub den: i64,
}

impl Dur {
    pub fn new(num: i64, den: i64) -> Self {
        assert_ne!(den, 0);
        return Dur { num: num, den: den };
    }

    pub fn recip(self) -> Self {
        Dur::new(self.den, self.num)
    }

    pub fn reduce(self) -> Self {
        let gcdiv = gcd(self.num, self.den);
        if gcdiv == 1 {
            return self;
        }
        Dur::new(self.num / gcdiv, self.den / gcdiv)
    }

    pub fn div_int(self, divisor: i64) -> Self {
        Dur::new(self.num, self.den * divisor)
    }
}

impl Add for Dur {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        assert_ne!(self.den, 0);
        assert_ne!(other.den, 0);
        let mul = lcm(self.den, other.den);
        Dur::new(
            (self.num * (mul / self.den)) + (other.num * (mul / other.den)),
            mul,
        )
    }
}

impl Sub for Dur {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        assert_ne!(self.den, 0);
        assert_ne!(other.den, 0);
        let mul = lcm(self.den, other.den);
        Dur::new(
            (self.num * (mul / self.den)) - (other.num * (mul / other.den)),
            mul,
        )
    }
}

impl Mul for Dur {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        assert_ne!(self.den, 0);
        assert_ne!(other.den, 0);
        Dur::new(self.num * other.num, self.den * other.den)
    }
}

impl Div for Dur {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        self.mul(other.recip())
    }
}

#[cfg(test)]
mod tests {
    use crate::dur::Dur;
    use std::panic;

    #[test]
    fn test_zero_denominator() {
        let result = panic::catch_unwind(|| {
            let _half = Dur::new(1, 0);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_dur_reduce() {
        let third = Dur::new(3, 9);
        assert_eq!(third.reduce(), Dur::new(1, 3));
    }

    #[test]
    fn test_dur_add() {
        let half = Dur::new(1, 2);
        let third = Dur::new(1, 3);
        assert_eq!(half + third, Dur::new(5, 6));
    }

    #[test]
    fn test_dur_sub() {
        let half = Dur::new(1, 2);
        let third = Dur::new(1, 3);
        assert_eq!(half - third, Dur::new(1, 6));
    }

    #[test]
    fn test_dur_mul() {
        let half = Dur::new(1, 2);
        let third = Dur::new(2, 7);
        assert_eq!(half * third, Dur::new(2, 14));
    }

    #[test]
    fn test_dur_div() {
        let half = Dur::new(1, 2);
        let third = Dur::new(1, 3);
        assert_eq!(half / third, Dur::new(3, 2));
    }

    #[test]
    fn test_dur_div_int() {
        let half = Dur::new(1, 2);
        assert_eq!(half.div_int(3 as i64), Dur::new(1, 6));
    }
}
