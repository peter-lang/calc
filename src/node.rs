use crate::error::CalcError;
use crate::number::Number;
use crate::unit::Unit;
use crate::value::Value;

#[derive(Clone)]
pub enum Node {
    Value(Value),
    UnaryExpr {
        op: fn(Value) -> Result<Value, CalcError>,
        val: Box<Node>,
    },
    BinaryExpr {
        op: fn(Value, Value) -> Result<Value, CalcError>,
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
            Node::UnaryExpr { op, val } => op(val.eval()?),
            Node::BinaryExpr { op, lhs, rhs } => op(lhs.eval()?, rhs.eval()?),
        }
    }
}
