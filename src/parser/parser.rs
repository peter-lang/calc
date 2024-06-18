use std::collections::HashMap;

use crate::{unit, value_op};
use crate::node::Node;
use crate::number::Number;
use crate::parser::token::{CURRENCIES, Token};
use crate::unit::Unit;
use crate::value::Value;

#[derive(Clone)]
enum Match<T> {
    Ok(T, usize),
    Err,
}

type MemoPos = (usize, &'static str);

pub struct Parser {
    tokens: Vec<Token>,
    memos: HashMap<MemoPos, Match<Node>>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            tokens: vec![],
            memos: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn extend(&mut self, tokens: impl IntoIterator<Item = Token>) {
        self.tokens.extend(tokens);
    }

    pub fn reset(&mut self) {
        self.tokens.clear();
    }

    pub fn parse(&mut self) -> Option<Node> {
        // TODO: we should be able to re-use the previous computations
        self.memos.clear();
        let pos_end = self.tokens.len();
        if let Match::Ok(node, pos) = self.expression(0) {
            if pos == pos_end {
                return Some(node);
            }
        }
        return None;
    }

    fn memoize(
        &mut self,
        pos: usize,
        name: &'static str,
        base: fn(&mut Self, usize) -> Match<Node>,
    ) -> Match<Node> {
        if pos >= self.tokens.len() {
            return Match::Err;
        }

        let key = (pos, name);
        if let Some(res) = self.memos.get(&key) {
            return res.clone();
        }

        let result = base(self, pos);
        self.memos.insert(key, result.clone());
        return result;
    }

    fn memoize_left_rec(
        &mut self,
        pos: usize,
        name: &'static str,
        base: fn(&mut Self, usize) -> Match<Node>,
    ) -> Match<Node> {
        if pos >= self.tokens.len() {
            return Match::Err;
        }

        let key = (pos, name);
        if let Some(res) = self.memos.get(&key) {
            return res.clone();
        }

        let mut last_result = Match::Err;
        self.memos.insert(key, Match::Err);
        while let Match::Ok(n, end_pos) = base(self, pos) {
            if let Match::Ok(_, prev_end_pos) = last_result {
                if end_pos <= prev_end_pos {
                    break;
                }
            }
            last_result = Match::Ok(n, end_pos);
            self.memos.insert(key, last_result.clone());
        }

        return last_result;
    }

    fn expect(&self, pos: usize, tok: Token) -> Option<usize> {
        if pos < self.tokens.len() && self.tokens[pos] == tok {
            return Some(pos + 1);
        }
        return None;
    }

    fn num_unit(&mut self, pos: usize) -> Match<Node> {
        self.memoize_left_rec(pos, "num_unit", Self::num_unit_inner)
    }

    fn num_unit_inner(&mut self, pos: usize) -> Match<Node> {
        if let Match::Ok(
            Node::Value(Value {
                num: lhs_num,
                unit: Some(lhs_unit),
            }),
            pos,
        ) = self.num_unit(pos)
        {
            if let Match::Ok((rhs_num, rhs_unit), pos) = self.expect_single_num_unit(pos) {
                if let Some(_) = unit::common_type(&lhs_unit, &rhs_unit) {
                    return Match::Ok(
                        Node::BinaryExpr {
                            op: value_op::add,
                            lhs: Box::new(Node::value(lhs_num, Some(lhs_unit))),
                            rhs: Box::new(Node::value(rhs_num, Some(rhs_unit))),
                        },
                        pos,
                    );
                }
            }
        }
        if let Match::Ok((num, unit), pos) = self.expect_single_num_unit(pos) {
            return Match::Ok(Node::value(num, Some(unit)), pos);
        }
        Match::Err
    }

    fn expect_single_num_unit(&mut self, pos: usize) -> Match<(Number, Unit)> {
        if let Match::Ok(num, pos) = self.expect_number(pos) {
            if let Match::Ok(unit, pos) = self.expect_unit(pos) {
                return Match::Ok((num, unit), pos);
            }
        }
        Match::Err
    }

    fn atom(&mut self, pos: usize) -> Match<Node> {
        self.memoize(pos, "atom", Self::atom_inner)
    }

    fn atom_inner(&mut self, pos: usize) -> Match<Node> {
        if let Some(pos) = self.expect(pos, Token::ParBegin) {
            if let Match::Ok(val, pos) = self.expression(pos) {
                if let Some(pos) = self.expect(pos, Token::ParEnd) {
                    return Match::Ok(val, pos);
                }
            }
        }
        if let Match::Ok(node, pos) = self.num_unit(pos) {
            return Match::Ok(node, pos);
        }
        if let Match::Ok(num, pos) = self.expect_number(pos) {
            return Match::Ok(Node::value(num, None), pos);
        }
        if let Match::Ok(unit, pos) = self.expect_unit(pos) {
            return Match::Ok(Node::value(1.into(), Some(unit)), pos);
        }
        Match::Err
    }

    fn exponent(&mut self, pos: usize) -> Match<Node> {
        self.memoize(pos, "exponent", Self::exponent_inner)
    }

    fn exponent_inner(&mut self, pos: usize) -> Match<Node> {
        if let Match::Ok(lhs, pos) = self.atom(pos) {
            if let Some(pos) = self.expect(pos, Token::Exp) {
                if let Match::Ok(rhs, pos) = self.exponent(pos) {
                    return Match::Ok(
                        Node::BinaryExpr {
                            op: value_op::pow,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        pos,
                    );
                }
            }
        }
        self.atom(pos)
    }

    fn term(&mut self, pos: usize) -> Match<Node> {
        self.memoize_left_rec(pos, "term", Self::term_inner)
    }

    fn term_inner(&mut self, pos: usize) -> Match<Node> {
        if let Match::Ok(lhs, pos) = self.term(pos) {
            if let Some(pos) = self.expect(pos, Token::Mul) {
                if let Match::Ok(rhs, pos) = self.exponent(pos) {
                    return Match::Ok(
                        Node::BinaryExpr {
                            op: value_op::mul,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        pos,
                    );
                }
            } else if let Some(pos) = self.expect(pos, Token::Div) {
                if let Match::Ok(rhs, pos) = self.exponent(pos) {
                    return Match::Ok(
                        Node::BinaryExpr {
                            op: value_op::div,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        pos,
                    );
                }
            } else if let Some(pos) = self.expect(pos, Token::KwTo) {
                if let Match::Ok(unit, pos) = self.expect_unit(pos) {
                    return Match::Ok(
                        Node::BinaryExpr {
                            op: value_op::conversion,
                            lhs: Box::new(lhs),
                            rhs: Box::new(Node::value(1.into(), Some(unit))),
                        },
                        pos,
                    );
                }
            }
        }
        self.exponent(pos)
    }

    fn expression(&mut self, pos: usize) -> Match<Node> {
        self.memoize_left_rec(pos, "expression", Self::expression_inner)
    }

    fn expression_inner(&mut self, pos: usize) -> Match<Node> {
        if let Match::Ok(lhs, pos) = self.expression(pos) {
            if let Some(pos) = self.expect(pos, Token::Add) {
                if let Match::Ok(rhs, pos) = self.term(pos) {
                    return Match::Ok(
                        Node::BinaryExpr {
                            op: value_op::add,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        pos,
                    );
                }
            } else if let Some(pos) = self.expect(pos, Token::Sub) {
                if let Match::Ok(rhs, pos) = self.term(pos) {
                    return Match::Ok(
                        Node::BinaryExpr {
                            op: value_op::sub,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        pos,
                    );
                }
            }
        }
        if let Some(pos) = self.expect(pos, Token::Sub) {
            if let Match::Ok(rhs, pos) = self.term(pos) {
                return Match::Ok(
                    Node::UnaryExpr {
                        op: value_op::sub_unary,
                        val: Box::new(rhs),
                    },
                    pos,
                );
            }
        }
        self.term(pos)
    }

    fn expect_number(&mut self, pos: usize) -> Match<Number> {
        if pos >= self.tokens.len() {
            return Match::Err;
        }
        let num: Option<Number> = match self.tokens[pos] {
            Token::LitInt(val) => Some(val.into()),
            Token::LitFloat(val) => Some(val.into()),
            _ => None,
        };
        if let Some(num) = num {
            Match::Ok(num, pos + 1)
        } else {
            Match::Err
        }
    }

    fn expect_unit(&mut self, pos: usize) -> Match<Unit> {
        if pos >= self.tokens.len() {
            return Match::Err;
        }
        let unit = match &self.tokens[pos] {
            Token::LenM => Some(Unit::LenM),
            Token::LenKm => Some(Unit::LenKm),
            Token::LenCm => Some(Unit::LenCm),
            Token::LenMm => Some(Unit::LenMm),
            Token::LenInch => Some(Unit::LenInch),
            Token::LenFeet => Some(Unit::LenFeet),
            Token::LenYard => Some(Unit::LenYard),
            Token::LenMile => Some(Unit::LenMile),
            Token::AreaM => Some(Unit::AreaM),
            Token::AreaKm => Some(Unit::AreaKm),
            Token::AreaCm => Some(Unit::AreaCm),
            Token::AreaMm => Some(Unit::AreaMm),
            Token::AreaInch => Some(Unit::AreaInch),
            Token::AreaFeet => Some(Unit::AreaFeet),
            Token::AreaYard => Some(Unit::AreaYard),
            Token::AreaMile => Some(Unit::AreaMile),
            Token::VolLiter => Some(Unit::VolLiter),
            Token::VolMilliLiter => Some(Unit::VolMilliLiter),
            Token::VolM => Some(Unit::VolM),
            Token::VolCm => Some(Unit::VolCm),
            Token::VolMm => Some(Unit::VolMm),
            Token::VolInch => Some(Unit::VolInch),
            Token::VolFeet => Some(Unit::VolFeet),
            Token::VolYard => Some(Unit::VolYard),
            Token::VolPint => Some(Unit::VolPint),
            Token::TempC => Some(Unit::TempC),
            Token::TempF => Some(Unit::TempF),
            Token::TimeHour => Some(Unit::TimeHour),
            Token::TimeMin => Some(Unit::TimeMin),
            Token::TimeSec => Some(Unit::TimeSec),
            Token::Curr(name) => {
                if let Ok(idx) = CURRENCIES.binary_search_by(|p: &&str| (*p).cmp(name.as_str())) {
                    Some(Unit::Curr(CURRENCIES[idx]))
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(unit) = unit {
            Match::Ok(unit, pos + 1)
        } else {
            Match::Err
        }
    }
}
