use crate::{number_op, unit};
use crate::error::CalcError;
use crate::value::Value;

#[derive(Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Conversion,
}

#[derive(Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,
}

impl BinaryOp {
    pub fn apply(self, lhs: Value, rhs: Value) -> Result<Value, CalcError> {
        match self {
            BinaryOp::Add => add(lhs, rhs),
            BinaryOp::Sub => sub(lhs, rhs),
            BinaryOp::Mul => mul(lhs, rhs),
            BinaryOp::Div => div(lhs, rhs),
            BinaryOp::Pow => pow(lhs, rhs),
            BinaryOp::Conversion => conversion(lhs, rhs),
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Pow => "^",
            BinaryOp::Conversion => "to",
        }
    }
}

impl UnaryOp {
    pub fn apply(self, val: Value) -> Result<Value, CalcError> {
        match self {
            UnaryOp::Neg => sub_unary(val),
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            UnaryOp::Neg => "-",
        }
    }
}

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
    if rhs.unit.is_some() {
        return Err(CalcError::ExpByUnit);
    }
    // Raising a united value to a power would produce a derived unit (e.g. m²),
    // which we don't support yet — reuse OperateWithUnits until unit algebra lands.
    if lhs.unit.is_some() {
        return Err(CalcError::OperateWithUnits);
    }
    Ok(Value {
        num: number_op::pow(lhs.num, rhs.num)?,
        unit: None,
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
