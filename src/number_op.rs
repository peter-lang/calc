use crate::error::CalcError;
use crate::number::Number;
use crate::rational::Rational;

pub fn pow(lhs: Number, rhs: Number) -> Result<Number, CalcError> {
    match (lhs, rhs) {
        (Number::Int(lhs), Number::Int(rhs)) => {
            if let Some(exp) = u32::try_from(rhs.unsigned_abs()).ok() {
                if let Some(res) = lhs.checked_pow(exp) {
                    return if rhs >= 0 {
                        Ok(Number::Int(res))
                    } else {
                        Ok(Number::Rational(Rational::inverse(res)?))
                    };
                }
            }
            if let Ok(rhs) = i32::try_from(rhs) {
                Ok(Number::Float((lhs as f64).powi(rhs)))
            } else {
                Ok(Number::Float((lhs as f64).powf(rhs as f64)))
            }
        }
        (Number::Float(lhs), Number::Int(rhs)) => {
            if let Ok(rhs) = i32::try_from(rhs) {
                Ok(Number::Float(lhs.powi(rhs)))
            } else {
                Ok(Number::Float(lhs.powf(rhs as f64)))
            }
        }
        (lhs, Number::Float(rhs)) => Ok(Number::Float(lhs.to_float().powf(rhs))),
        (Number::Rational(lhs), Number::Int(rhs)) => {
            if let Ok(rhs) = i32::try_from(rhs) {
                if let Some(res) = lhs.checked_pow(rhs) {
                    Ok(Number::Rational(res))
                } else {
                    let lhs: f64 = lhs.into();
                    Ok(Number::Float(lhs.powi(rhs)))
                }
            } else {
                let lhs: f64 = lhs.into();
                Ok(Number::Float(lhs.powf(rhs as f64)))
            }
        }
        (lhs, rhs) => Ok(Number::Float(lhs.to_float().powf(rhs.to_float()))),
    }
}

pub fn mul(lhs: Number, rhs: Number) -> Number {
    match (lhs, rhs) {
        (Number::Int(lhs), Number::Int(rhs)) => Number::Int(lhs * rhs),
        (Number::Float(lhs), rhs) => Number::Float(lhs * rhs.to_float()),
        (lhs, Number::Float(rhs)) => Number::Float(lhs.to_float() * rhs),
        (lhs, rhs) => {
            let res: Rational = lhs.to_rational() * rhs.to_rational();
            if res.den == 1 {
                Number::Int(res.num)
            } else {
                Number::Rational(res)
            }
        }
    }
}

pub fn div(lhs: Number, rhs: Number) -> Result<Number, CalcError> {
    match (lhs, rhs) {
        (Number::Float(lhs), rhs) => {
            let rhs = rhs.to_float();
            if rhs == 0.0 {
                return Err(CalcError::DivByZero);
            }
            Ok(Number::Float(lhs / rhs))
        }
        (lhs, Number::Float(rhs)) => {
            if rhs == 0.0 {
                return Err(CalcError::DivByZero);
            }
            Ok(Number::Float(lhs.to_float() / rhs))
        }
        (lhs, rhs) => {
            let rhs = rhs.to_rational();
            if rhs == 0 {
                return Err(CalcError::DivByZero);
            }
            let lhs = lhs.to_rational();
            let res = lhs / rhs;
            if res.den == 1 {
                Ok(Number::Int(res.num))
            } else {
                Ok(Number::Rational(res))
            }
        }
    }
}

pub fn add(lhs: Number, rhs: Number) -> Number {
    match (lhs, rhs) {
        (Number::Int(lhs), Number::Int(rhs)) => Number::Int(lhs + rhs),
        (Number::Float(lhs), rhs) => Number::Float(lhs + rhs.to_float()),
        (lhs, Number::Float(rhs)) => Number::Float(lhs.to_float() + rhs),
        (lhs, rhs) => {
            let res: Rational = lhs.to_rational() + rhs.to_rational();
            if res.den == 1 {
                Number::Int(res.num)
            } else {
                Number::Rational(res)
            }
        }
    }
}

pub fn sub(lhs: Number, rhs: Number) -> Number {
    match (lhs, rhs) {
        (Number::Int(lhs), Number::Int(rhs)) => Number::Int(lhs - rhs),
        (Number::Float(lhs), rhs) => Number::Float(lhs - rhs.to_float()),
        (lhs, Number::Float(rhs)) => Number::Float(lhs.to_float() - rhs),
        (lhs, rhs) => {
            let res: Rational = lhs.to_rational() - rhs.to_rational();
            if res.den == 1 {
                Number::Int(res.num)
            } else {
                Number::Rational(res)
            }
        }
    }
}

pub fn sub_unary(val: Number) -> Number {
    match val {
        Number::Int(val) => Number::Int(-val),
        Number::Float(val) => Number::Float(-val),
        Number::Rational(val) => Number::Rational(-val),
    }
}
