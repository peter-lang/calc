use crate::error::CalcError;
use crate::number::Number;
use crate::unit::Unit;
use crate::value::Value;
use crate::value_op::{BinaryOp, UnaryOp};

#[derive(Clone)]
pub enum Node {
    Value(Value),
    UnaryExpr {
        op: UnaryOp,
        val: Box<Node>,
    },
    BinaryExpr {
        op: BinaryOp,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
}

impl Node {
    pub fn value(num: Number, unit: Option<Unit>) -> Node {
        return Node::Value(Value { num, unit });
    }

    pub fn eval(self) -> Result<Value, CalcError> {
        match self {
            Node::Value(val) => Ok(val),
            Node::UnaryExpr { op, val } => op.apply(val.eval()?),
            Node::BinaryExpr { op, lhs, rhs } => op.apply(lhs.eval()?, rhs.eval()?),
        }
    }
}
