pub mod common;
pub mod input;
pub mod keyword;
pub mod position;
pub mod problem;
pub mod symbol;

use std::fmt::{self, Debug};

use num::Integer;
use position::{EqNoPosition, Position};
use symbol::{Symbol, TSymbol};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Q<T> {
    pub path: Vec<Symbol>,
    pub types: Vec<TSymbol>,
    pub symbol: T,
}

impl<T> Q<T> {
    pub fn of(symbol: T) -> Q<T> {
        Q {
            path: Vec::default(),
            types: Vec::default(),
            symbol,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    Type(Q<TSymbol>),
}

pub type ERef = Box<Expression>;
pub type Unary = common::Unary<ERef>;
pub type Binary = common::Binary<ERef>;
pub type Seq = common::Seq<ERef>;
pub type Conditional = common::Conditional<ERef>;

#[derive(Debug)]
pub enum Expr {
    LitInteger(Integer),
    Binary(Binary),
    Symbol(Q<Symbol>),
    Type(Q<TSymbol>),
}

impl Expr {
    pub fn to_expression(self, position: Position, type_expr: Option<TypeExpr>) -> Expression {
        Expression {
            position,
            expr: self,
            type_expr,
        }
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

pub struct Expression {
    pub position: Position,
    pub expr: Expr,
    pub type_expr: Option<TypeExpr>,
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
pub enum Statement {
    Expression(Expression),
}

impl EqNoPosition for Statement {
    fn eq_nopos(&self, other: &Self) -> bool {
        match self {
            Statement::Expression(e1) => match other {
                Statement::Expression(e2) => e1.eq_nopos(e2),
                _ => false,
            },
        }
    }
}

#[derive(Debug)]
pub struct AST {
    pub expression: Option<Expression>,
}
