use std::fmt::{self, Debug};

use ast::position::{EqNoPosition, Position};
use ast::symbol::{Symbol, TSymbol};

use crate::resolved;
use crate::types::{TypeExpr, Value};

#[derive(Debug)]
pub(crate) enum Expr {
    Value(Value), // TODO
}

impl Expr {
    pub(crate) fn wrap(self, position: Position, type_expr: TypeExpr) -> Expression {
        Expression {
            position,
            expr: self,
            type_expr,
        }
    }

    pub(crate) fn wrap_from(
        self,
        expression: &resolved::Expression,
        type_expr: TypeExpr,
    ) -> Expression {
        self.wrap(expression.position.clone(), type_expr)
    }
}

impl EqNoPosition for Expr {
    fn eq_nopos(&self, other: &Self) -> bool {
        match self {
            Expr::Value(n1) => {
                if let Expr::Value(n2) = other {
                    n1 == n2
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
    pub(crate) type_expr: TypeExpr,
}

impl EqNoPosition for Expression {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.expr.eq_nopos(&other.expr) && self.type_expr == other.type_expr
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}[{:?}]{}", self.expr, self.type_expr, self.position)
    }
}

#[derive(Debug)]
pub(crate) struct TypeChecked {
    pub(crate) expression: Option<Expression>,
}
