pub mod input;
pub mod keyword;
pub mod position;
pub mod problem;
pub mod stmt;
pub mod symbol;

use std::{fmt, sync::Arc};

use num::Integer;
use position::{EqNoPosition, Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Minus,
    Not,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NEq,
    GT,
    GE,
    LT,
    LE,
    And,
    Or,
}

#[derive(Debug)]
pub struct Unary<P: Payload> {
    pub op: UnaryOp,
    pub expr: Expression<P>,
}

impl<P: Payload> EqNoPosition for Unary<P> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.op == other.op && self.expr.eq_nopos(&other.expr)
    }
}

#[derive(Debug)]
pub struct Binary<P: Payload> {
    pub op: BinaryOp,
    pub expr1: Expression<P>,
    pub expr2: Expression<P>,
}

impl<P: Payload> EqNoPosition for Binary<P> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.op == other.op
            && self.expr1.eq_nopos(&other.expr1)
            && self.expr2.eq_nopos(&other.expr2)
    }
}

#[derive(Debug)]
pub struct Seq<P: Payload> {
    pub expr: Expression<P>,
    pub then: Expression<P>,
}

impl<P: Payload> EqNoPosition for Seq<P> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.expr.eq_nopos(&other.expr) && self.then.eq_nopos(&other.then)
    }
}

#[derive(Debug)]
pub struct Conditional<P: Payload> {
    pub expr: Expression<P>,
    pub then: Expression<P>,
    pub otherwise: Expression<P>,
}

impl<P: Payload> EqNoPosition for Conditional<P> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.expr.eq_nopos(&other.expr)
            && self.then.eq_nopos(&other.then)
            && self.otherwise.eq_nopos(&other.otherwise)
    }
}

pub trait Payload: fmt::Debug + PartialEq + Eq {}

// Empty payload
pub struct Empty {}

impl fmt::Display for Empty {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl fmt::Debug for Empty {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl PartialEq for Empty {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for Empty {}
impl Payload for Empty {}

pub const EMPTY: Empty = Empty {};

trait Data: fmt::Debug + EqNoPosition {}

#[derive(Debug, PartialEq, Eq)]
struct Node<D: Data, P: Payload> {
    pub position: Position,
    pub data: D,
    pub payload: P,
}

impl<D: Data, P: Payload> EqNoPosition for Node<D, P> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.data.eq_nopos(&other.data) && self.payload == other.payload
    }
}

#[derive(Debug)]
struct ExprData<P: Payload> {
    position: Position,
    expr: Expr<P>,
    payload: P,
}

#[derive(Debug)]
pub enum Expr<P: Payload> {
    LitInteger(Integer),
    Binary(Binary<P>),
}

impl<P: Payload> Expr<P> {
    pub fn to_expression(self, position: Position, payload: P) -> Expression<P> {
        Expression::new(position, self, payload)
    }
}

impl<P: Payload> Data for Expr<P> {}

impl<P: Payload> EqNoPosition for Expr<P> {
    fn eq_nopos(&self, other: &Self) -> bool {
        match self {
            Expr::LitInteger(n1) => {
                if let Expr::LitInteger(n2) = other {
                    n1 == n2
                } else {
                    false
                }
            }
            Expr::Binary(b1) => {
                if let Expr::Binary(b2) = other {
                    b1.eq_nopos(b2)
                } else {
                    false
                }
            }
        }
    }
}

pub struct Expression<P: Payload> {
    data: Arc<ExprData<P>>,
}

impl<P: Payload> Expression<P> {
    pub fn new(position: Position, expr: Expr<P>, payload: P) -> Self {
        Self {
            data: Arc::new(ExprData {
                position,
                expr,
                payload,
            }),
        }
    }

    pub fn position(&self) -> &Position {
        &self.data.position
    }

    pub fn expr(&self) -> &Expr<P> {
        &self.data.expr
    }

    pub fn payload(&self) -> &P {
        &self.data.payload
    }
}

impl<P: Payload> EqNoPosition for Expression<P> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.data.expr.eq_nopos(&other.data.expr) && self.data.payload == other.data.payload
    }
}

impl<P: Payload> fmt::Debug for Expression<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}[{:?}]{}",
            self.data.expr, self.data.payload, self.data.position
        )
    }
}

#[derive(Debug)]
pub struct AST<T: Payload> {
    pub expression: Option<Expression<T>>,
}
