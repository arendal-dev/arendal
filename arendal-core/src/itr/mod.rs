use std::fmt::{self, Debug};

use ast::position::{EqNoPosition, Position};
use ast::symbol::{Symbol, TSymbol};
use num::Integer;

pub(crate) trait Payload: fmt::Debug + PartialEq + Eq {}

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

pub(crate) const EMPTY: Empty = Empty {};

impl<T: Payload> Payload for Option<T> {}

pub(crate) type Unary<P: Payload> = ast::common::Unary<Expression<P>>;
pub(crate) type Binary<P: Payload> = ast::common::Binary<Expression<P>>;
pub(crate) type Seq<P: Payload> = ast::common::Seq<Expression<P>>;
pub(crate) type Conditional<P: Payload> = ast::common::Conditional<Expression<P>>;

#[derive(Debug)]
struct ExprData<P: Payload> {
    position: Position,
    expr: Expr<P>,
    payload: P,
}

#[derive(Debug)]
pub(crate) enum Expr<P: Payload> {
    LitInteger(Integer),
    Binary(Binary<P>),
    Symbol(Symbol),
    Type(TSymbol),
}

impl<P: Payload> Expr<P> {
    pub fn to_expression(self, position: Position, payload: P) -> Expression<P> {
        Expression::new(position, self, payload)
    }
}

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
            _ => panic!("TODO!"),
        }
    }
}

pub(crate) struct Expression<P: Payload> {
    data: Box<ExprData<P>>,
}

impl<P: Payload> Expression<P> {
    pub fn new(position: Position, expr: Expr<P>, payload: P) -> Self {
        Self {
            data: Box::new(ExprData {
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
pub(crate) struct ITR<P: Payload> {
    pub expression: Option<Expression<P>>,
}
