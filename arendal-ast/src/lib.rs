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

pub type Unary = common::Unary<Expression>;
pub type Binary = common::Binary<Expression>;
pub type Seq = common::Seq<Expression>;
pub type Conditional = common::Conditional<Expression>;

#[derive(Debug)]
struct ExprData {
    position: Position,
    expr: Expr,
    type_annotation: Option<TypeExpr>,
}

#[derive(Debug)]
pub enum Expr {
    LitInteger(Integer),
    Binary(Binary),
    Symbol(Q<Symbol>),
    Type(Q<TSymbol>),
}

impl Expr {
    pub fn to_expression(
        self,
        position: Position,
        type_annotation: Option<TypeExpr>,
    ) -> Expression {
        Expression::new(position, self, type_annotation)
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
    data: Box<ExprData>,
}

impl Expression {
    pub fn new(position: Position, expr: Expr, type_annotation: Option<TypeExpr>) -> Self {
        Self {
            data: Box::new(ExprData {
                position,
                expr,
                type_annotation,
            }),
        }
    }

    pub fn position(&self) -> &Position {
        &self.data.position
    }

    pub fn expr(&self) -> &Expr {
        &self.data.expr
    }

    pub fn annotate(self, type_annotation: TypeExpr) -> Self {
        Self::new(self.data.position, self.data.expr, Some(type_annotation))
    }
}

impl EqNoPosition for Expression {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.data.expr.eq_nopos(&other.data.expr)
            && self.data.type_annotation == other.data.type_annotation
    }
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}[{:?}]{}",
            self.data.expr, self.data.type_annotation, self.data.position
        )
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
