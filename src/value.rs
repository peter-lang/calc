use std::fmt::{Display, Formatter};

use crate::number::Number;
use crate::unit::{get_unit_name, Unit};

#[derive(Clone)]
pub struct Value {
    pub num: Number,
    pub unit: Option<Unit>,
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(unit) = &self.unit {
            write!(f, "{}{}", self.num, get_unit_name(unit))
        } else {
            write!(f, "{}", self.num)
        }
    }
}

impl From<i64> for Value {
    fn from(item: i64) -> Self {
        Value {
            num: Number::from(item),
            unit: None,
        }
    }
}

impl From<f64> for Value {
    fn from(item: f64) -> Self {
        Value {
            num: Number::from(item),
            unit: None,
        }
    }
}

impl From<Unit> for Value {
    fn from(unit: Unit) -> Self {
        Value {
            num: Number::from(1i64),
            unit: Some(unit),
        }
    }
}
