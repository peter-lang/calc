use std::fmt::{Debug, Formatter};

use crate::error::CalcError;
use crate::node::Node;
use crate::number::Number;
use crate::value::Value;
use crate::value_op::{add, div, mul, pow, sub, sub_unary};

impl Debug for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub fn fmt_unary_op(
    f: &mut Formatter,
    fun: fn(Value) -> Result<Value, CalcError>,
) -> std::fmt::Result {
    if fun == sub_unary {
        write!(f, "-")
    } else {
        write!(f, "?")
    }
}

pub fn fmt_binary_op(
    f: &mut Formatter,
    fun: fn(Value, Value) -> Result<Value, CalcError>,
) -> std::fmt::Result {
    if fun == pow {
        write!(f, "^")
    } else if fun == mul {
        write!(f, "*")
    } else if fun == div {
        write!(f, "/")
    } else if fun == add {
        write!(f, "+")
    } else if fun == sub {
        write!(f, "-")
    } else {
        write!(f, "?")
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Value(x) => write!(f, "{}", x),
            Node::UnaryExpr { op, val } => {
                write!(f, "(")?;
                fmt_unary_op(f, *op)?;
                write!(f, "{:?}", val)?;
                write!(f, ")")
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                write!(f, "(")?;
                write!(f, "{:?}", lhs)?;
                fmt_binary_op(f, *op)?;
                write!(f, "{:?}", rhs)?;
                write!(f, ")")
            }
        }
    }
}
