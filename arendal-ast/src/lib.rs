pub mod input;
pub mod keyword;
pub mod position;
pub mod problem;
pub mod stmt;
pub mod symbol;

use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use num::Integer;
use position::{EqNoPosition, Position};
use symbol::{Symbol, TSymbol};

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
pub struct Unary<T: Payload, P: Payload, Q: Debug, QT: Debug> {
    pub op: UnaryOp,
    pub expr: Expression<T, P, Q, QT>,
}

impl<T: Payload, P: Payload, Q: Debug, QT: Debug> EqNoPosition for Unary<T, P, Q, QT> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.op == other.op && self.expr.eq_nopos(&other.expr)
    }
}

#[derive(Debug)]
pub struct Binary<T: Payload, P: Payload, Q: Debug, QT: Debug> {
    pub op: BinaryOp,
    pub expr1: Expression<T, P, Q, QT>,
    pub expr2: Expression<T, P, Q, QT>,
}

impl<T: Payload, P: Payload, Q: Debug, QT: Debug> EqNoPosition for Binary<T, P, Q, QT> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.op == other.op
            && self.expr1.eq_nopos(&other.expr1)
            && self.expr2.eq_nopos(&other.expr2)
    }
}

#[derive(Debug)]
pub struct Seq<T: Payload, P: Payload, Q: Debug, QT: Debug> {
    pub expr: Expression<T, P, Q, QT>,
    pub then: Expression<T, P, Q, QT>,
}

impl<T: Payload, P: Payload, Q: Debug, QT: Debug> EqNoPosition for Seq<T, P, Q, QT> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.expr.eq_nopos(&other.expr) && self.then.eq_nopos(&other.then)
    }
}

#[derive(Debug)]
pub struct Conditional<T: Payload, P: Payload, Q: Debug, QT: Debug> {
    pub expr: Expression<T, P, Q, QT>,
    pub then: Expression<T, P, Q, QT>,
    pub otherwise: Expression<T, P, Q, QT>,
}

impl<T: Payload, P: Payload, Q: Debug, QT: Debug> EqNoPosition for Conditional<T, P, Q, QT> {
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
    position: Position,
    data: D,
    payload: P,
}

impl<D: Data, P: Payload> EqNoPosition for Node<D, P> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.data.eq_nopos(&other.data) && self.payload == other.payload
    }
}

#[derive(Debug)]
struct ExprData<T: Payload, P: Payload, Q: Debug, QT: Debug> {
    position: Position,
    expr: Expr<T, P, Q, QT>,
    type_annotation: T,
    payload: P,
}

#[derive(Debug)]
pub enum SymRef<L, Q> {
    Local(L),
    Qualified(Q),
}

pub type SymbolRef<Q> = SymRef<Symbol, Q>;
pub type TSymbolRef<Q> = SymRef<TSymbol, Q>;

#[derive(Debug)]
pub enum Expr<T: Payload, P: Payload, Q: Debug, QT: Debug> {
    LitInteger(Integer),
    Binary(Binary<T, P, Q, QT>),
    SymbolRef(SymbolRef<Q>),
    TypeRef(TSymbolRef<QT>),
}

impl<T: Payload, P: Payload, Q: Debug, QT: Debug> Expr<T, P, Q, QT> {
    pub fn to_expression(
        self,
        position: Position,
        type_annotation: T,
        payload: P,
    ) -> Expression<T, P, Q, QT> {
        Expression::new(position, self, type_annotation, payload)
    }
}

impl<T: Payload, P: Payload, Q: Debug, QT: Debug> Data for Expr<T, P, Q, QT> {}

impl<T: Payload, P: Payload, Q: Debug, QT: Debug> EqNoPosition for Expr<T, P, Q, QT> {
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
            _ => panic!("TODO!"),
        }
    }
}

pub struct Expression<T: Payload, P: Payload, Q: Debug, QT: Debug> {
    data: Arc<ExprData<T, P, Q, QT>>,
}

impl<T: Payload, P: Payload, Q: Debug, QT: Debug> Expression<T, P, Q, QT> {
    pub fn new(
        position: Position,
        expr: Expr<T, P, Q, QT>,
        type_annotation: T,
        payload: P,
    ) -> Self {
        Self {
            data: Arc::new(ExprData {
                position,
                expr,
                type_annotation,
                payload,
            }),
        }
    }

    pub fn position(&self) -> &Position {
        &self.data.position
    }

    pub fn expr(&self) -> &Expr<T, P, Q, QT> {
        &self.data.expr
    }

    pub fn payload(&self) -> &P {
        &self.data.payload
    }
}

impl<T: Payload, P: Payload, Q: Debug, QT: Debug> EqNoPosition for Expression<T, P, Q, QT> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.data.expr.eq_nopos(&other.data.expr) && self.data.payload == other.data.payload
    }
}

impl<T: Payload, P: Payload, Q: Debug, QT: Debug> fmt::Debug for Expression<T, P, Q, QT> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}[{:?}]{}",
            self.data.expr, self.data.payload, self.data.position
        )
    }
}

#[derive(Debug)]
pub struct AST<T: Payload, P: Payload, Q: Debug, QT: Debug> {
    pub expression: Option<Expression<T, P, Q, QT>>,
}
