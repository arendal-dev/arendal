use std::fmt::{self, Debug};

use ast::position::{EqNoPosition, Position};
use ast::symbol::{FQPath, FQSym, FQType, Symbol};
use num::Integer;

pub(crate) type ERef = Box<Expression>;

pub(crate) type Unary = ast::common::Unary<ERef>;
pub(crate) type Binary = ast::common::Binary<ERef>;
pub(crate) type Seq = ast::common::Seq<ERef>;
pub(crate) type Conditional = ast::common::Conditional<ERef>;

#[derive(Debug)]
pub(crate) enum Expr {
    LitInteger(Integer),
    Binary(Binary),
    LocalSymbol(Symbol),
    Symbol(FQSym),
    Type(FQType),
}

impl Expr {
    pub(crate) fn wrap(self, position: Position) -> Expression {
        Expression {
            position,
            expr: self,
        }
    }

    pub(crate) fn wrap_from(self, expression: &ast::Expression) -> Expression {
        self.wrap(expression.position.clone())
    }
}

impl EqNoPosition for Expr {
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

pub(crate) struct Expression {
    pub(crate) position: Position,
    pub(crate) expr: Expr,
}

impl EqNoPosition for Expression {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.expr.eq_nopos(&other.expr)
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}{}", self.expr, self.position)
    }
}

#[derive(Debug)]
pub(crate) struct Resolved {
    pub(crate) path: FQPath,
    pub(crate) expression: Option<Expression>,
}
