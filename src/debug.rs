use std::fmt::{Debug, Formatter};

use crate::node::Node;
use crate::number::Number;

impl Debug for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Value(x) => write!(f, "{}", x),
            Node::UnaryExpr { op, val } => write!(f, "({}{:?})", op.symbol(), val),
            Node::BinaryExpr { op, lhs, rhs } => {
                write!(f, "({:?}{}{:?})", lhs, op.symbol(), rhs)
            }
        }
    }
}
