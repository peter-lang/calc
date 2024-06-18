use std::fmt::{Display, Formatter};
use std::ops;

use crate::error::CalcError;

#[derive(Clone, Debug)]
pub struct Rational {
    pub num: i64,
    pub den: u64,
}

impl Display for Rational {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.num, self.den)
    }
}

impl Rational {
    pub const fn new(num: i64, den: u64) -> Self {
        if den == 0 {
            panic!("Cannot divide by zero")
        }
        let _gcd = gcd(num.unsigned_abs(), den);
        Rational {
            num: num / (_gcd as i64),
            den: den / _gcd,
        }
    }

    pub fn inverse(value: i64) -> Result<Rational, CalcError> {
        if value == 0 {
            return Err(CalcError::DivByZero);
        }
        Ok(Rational {
            num: value.signum(),
            den: value.unsigned_abs(),
        })
    }

    pub fn invert(&self) -> Result<Rational, CalcError> {
        if self.num == 0 {
            return Err(CalcError::DivByZero);
        }
        let num = self.num.signum() * (self.den as i64);
        let den = self.num.unsigned_abs();
        Ok(Rational { num, den })
    }

    pub fn checked_pow(&self, value: i32) -> Option<Rational> {
        let abs = value.unsigned_abs();
        if let Some(num) = self.num.checked_pow(abs) {
            if let Some(den) = self.den.checked_pow(abs) {
                if value >= 0 {
                    return Some(Rational { num, den });
                } else {
                    if let Ok(den) = i64::try_from(den) {
                        return Some(Rational {
                            num: num.signum() * den,
                            den: num.unsigned_abs(),
                        });
                    }
                }
            }
        }
        None
    }

    pub const fn pow(&self, value: i32) -> Rational {
        let abs = value.unsigned_abs();
        let num = self.num.pow(abs);
        let den = self.den.pow(abs);
        if value >= 0 {
            Rational { num, den }
        } else {
            Rational {
                num: num.signum() * (den as i64),
                den: num.unsigned_abs(),
            }
        }
    }
}

impl From<i64> for Rational {
    fn from(value: i64) -> Self {
        Rational { num: value, den: 1 }
    }
}

impl Into<f64> for &Rational {
    fn into(self) -> f64 {
        (self.num as f64) / (self.den as f64)
    }
}

impl Into<f64> for Rational {
    fn into(self) -> f64 {
        (self.num as f64) / (self.den as f64)
    }
}

impl PartialEq<i64> for Rational {
    fn eq(&self, other: &i64) -> bool {
        self.den == 1 && self.num == *other
    }
}

impl PartialEq<Rational> for Rational {
    fn eq(&self, other: &Rational) -> bool {
        self.den == other.den && self.num == other.num
    }
}

const fn gcd(mut a: u64, mut b: u64) -> u64 {
    let mut r = a % b;
    while r > 0 {
        a = b % r;
        b = r;
        r = a;
    }
    b
}

fn lcm(a: u64, b: u64) -> u64 {
    a * b / gcd(a, b)
}

impl ops::Add<Rational> for Rational {
    type Output = Rational;

    fn add(self, rhs: Rational) -> Self::Output {
        let den = lcm(self.den, rhs.den);
        let num = self.num * ((den / self.den) as i64) + rhs.num * ((den / rhs.den) as i64);
        let _gcd = gcd(num.unsigned_abs(), den);
        Rational {
            num: num / (_gcd as i64),
            den: den / _gcd,
        }
    }
}

impl ops::Sub<Rational> for Rational {
    type Output = Rational;

    fn sub(self, rhs: Rational) -> Self::Output {
        let den = lcm(self.den, rhs.den);
        let num = self.num * ((den / self.den) as i64) - rhs.num * ((den / rhs.den) as i64);
        let _gcd = gcd(num.unsigned_abs(), den);
        Rational {
            num: num / (_gcd as i64),
            den: den / _gcd,
        }
    }
}

impl ops::Neg for Rational {
    type Output = Rational;

    fn neg(self) -> Self::Output {
        Rational {
            num: -self.num,
            den: self.den,
        }
    }
}

impl ops::Mul<Rational> for Rational {
    type Output = Rational;

    fn mul(self, rhs: Rational) -> Self::Output {
        let num = self.num * rhs.num;
        let den = self.den * rhs.den;
        let _gcd = gcd(num.unsigned_abs(), den);
        Rational {
            num: num / (_gcd as i64),
            den: den / _gcd,
        }
    }
}

impl ops::Div<Rational> for Rational {
    type Output = Rational;

    fn div(self, rhs: Rational) -> Self::Output {
        let num = self.num * rhs.num.signum() * (rhs.den as i64);
        let den = self.den * rhs.num.unsigned_abs();
        let _gcd = gcd(num.unsigned_abs(), den);
        Rational {
            num: num / (_gcd as i64),
            den: den / _gcd,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let result = Rational::new(1, 2) + Rational::new(1, 3);
        assert_eq!(result, Rational::new(5, 6));
    }

    #[test]
    fn test_sub_to_zero() {
        let result = Rational::new(1, 2) - Rational::new(1, 2);
        assert_eq!(result, Rational::new(0, 1));
    }

    #[test]
    fn test_mul_to_zero() {
        let result = Rational::new(1, 2) * Rational::new(0, 1);
        assert_eq!(result, Rational::new(0, 1));
    }
}
