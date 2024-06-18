use std::fmt::{Display, Formatter};

use crate::rational::Rational;

#[derive(Clone, PartialEq)]
pub enum Number {
    Int(i64),
    Rational(Rational),
    Float(f64),
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Int(x) => write!(f, "{}", x),
            Number::Rational(x) => write!(f, "{}", x),
            Number::Float(x) => write!(f, "{}", x),
        }
    }
}

impl From<i64> for Number {
    fn from(item: i64) -> Self {
        Number::Int(item)
    }
}

impl From<f64> for Number {
    fn from(item: f64) -> Self {
        Number::Float(item)
    }
}

impl Number {
    pub fn to_float(self) -> f64 {
        match self {
            Number::Int(x) => x as f64,
            Number::Rational(rat) => rat.into(),
            Number::Float(x) => x,
        }
    }

    pub fn to_rational(self) -> Rational {
        match self {
            Number::Int(x) => x.into(),
            Number::Rational(rat) => rat,
            Number::Float(x) => panic!("Cannot cast float {x} to rational"),
        }
    }
}
