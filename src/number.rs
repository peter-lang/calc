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
        fn float_fmt(res: f64, f: &mut Formatter<'_>) -> std::fmt::Result {
            let res_abs = res.abs();
            if res_abs >= 1e9 || res_abs < 1e-3 {
                write!(f, "{:.6e}", res)
            } else if res_abs >= 1e6 {
                let rounded = (res / 1e3).round() / 1e3;
                let postfix = if rounded == res / 1e6 { "" } else { ".." };
                write!(f, "{}{}m", rounded, postfix)
            } else if res_abs >= 1e3 {
                let rounded = res.round() / 1e3;
                let postfix = if rounded == res / 1e3 { "" } else { ".." };
                write!(f, "{}{}k", rounded, postfix)
            } else if res_abs >= 1. {
                let rounded = (res * 1e3).round() / 1e3;
                let postfix = if rounded == res { "" } else { ".." };
                write!(f, "{}{}", rounded, postfix)
            } else {
                let rounded = (res * 1e6).round() / 1e6;
                let postfix = if rounded == res { "" } else { ".." };
                write!(f, "{}{}", rounded, postfix)
            }
        }
        match self {
            Number::Int(res) => {
                let res_abs = res.abs();
                if res_abs >= 1_000_000 && res_abs % 1_000_000 == 0 {
                    write!(f, "{}m", res / 1_000_000)
                } else if res_abs >= 1_000 && res_abs % 1_000 == 0 {
                    write!(f, "{}k", res / 1_000)
                } else {
                    write!(f, "{}", res)
                }
            }
            Number::Rational(x) => {
                let res: f64 = x.into();
                float_fmt(res, f)
            }
            Number::Float(res) => float_fmt(*res, f),
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
