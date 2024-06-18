use crate::{number_op, unit};
use crate::error::CalcError;
use crate::value::Value;

pub fn conversion(lhs: Value, rhs: Value) -> Result<Value, CalcError> {
    if let Some(lhs_unit) = &lhs.unit {
        if let Some(rhs_unit) = &rhs.unit {
            return Ok(Value {
                num: unit::convert(lhs.num, lhs_unit, rhs_unit)?,
                unit: rhs.unit,
            });
        }
    }
    Err(CalcError::MissingUnit)
}

pub fn pow(lhs: Value, rhs: Value) -> Result<Value, CalcError> {
    if let Some(_) = rhs.unit {
        return Err(CalcError::ExpByUnit);
    }
    Ok(Value {
        num: number_op::pow(lhs.num, rhs.num)?,
        unit: lhs.unit,
    })
}

pub fn mul(lhs: Value, rhs: Value) -> Result<Value, CalcError> {
    Ok(Value {
        num: number_op::mul(lhs.num, rhs.num),
        unit: unit::single(lhs.unit, rhs.unit)?,
    })
}

pub fn div(lhs: Value, rhs: Value) -> Result<Value, CalcError> {
    Ok(Value {
        num: number_op::div(lhs.num, rhs.num)?,
        unit: unit::single(lhs.unit, rhs.unit)?,
    })
}

pub fn add(lhs: Value, rhs: Value) -> Result<Value, CalcError> {
    if lhs.unit == rhs.unit {
        return Ok(Value {
            num: number_op::add(lhs.num, rhs.num),
            unit: lhs.unit,
        });
    }
    if let Some(lhs_unit) = &lhs.unit {
        if let Some(rhs_unit) = &rhs.unit {
            let converted = unit::convert(rhs.num, rhs_unit, lhs_unit)?;
            return Ok(Value {
                num: number_op::add(lhs.num, converted),
                unit: lhs.unit,
            });
        }
    }
    Err(CalcError::DifferentUnitTypes)
}

pub fn sub(lhs: Value, rhs: Value) -> Result<Value, CalcError> {
    if lhs.unit == rhs.unit {
        return Ok(Value {
            num: number_op::sub(lhs.num, rhs.num),
            unit: lhs.unit,
        });
    }
    if let Some(lhs_unit) = &lhs.unit {
        if let Some(rhs_unit) = &rhs.unit {
            let converted = unit::convert(rhs.num, rhs_unit, lhs_unit)?;
            return Ok(Value {
                num: number_op::sub(lhs.num, converted),
                unit: lhs.unit,
            });
        }
    }
    Err(CalcError::DifferentUnitTypes)
}

pub fn sub_unary(val: Value) -> Result<Value, CalcError> {
    Ok(Value {
        num: number_op::sub_unary(val.num),
        unit: val.unit,
    })
}
