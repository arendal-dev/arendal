use std::rc::Rc;

use crate::{
    position::Position,
    symbol::{Symbol, TSymbol},
};

use num::Integer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    Symbol(Symbol),
    Type(TSymbol),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Q<T> {
    pub segments: Vec<Segment>,
    pub symbol: T,
}

pub type Unary = crate::common::Unary<Expression>;
pub type Binary = crate::common::Binary<Expression>;
pub type Seq = crate::common::Seq<Expression>;
pub type Conditional = crate::common::Conditional<Expression>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub exprs: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    LitInteger(Integer),
    Symbol(Q<Symbol>),
    TSymbol(Q<TSymbol>),
    Unary(Unary),
    Binary(Binary),
    Block(Block),
    Conditional(Conditional),
    Seq(Seq),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExprData {
    position: Position,
    expr: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    expr: Rc<ExprData>,
}

impl Expression {
    pub fn new(position: Position, expr: Expr) -> Self {
        Self {
            expr: Rc::new(ExprData { position, expr }),
        }
    }

    pub fn position(&self) -> &Position {
        &self.expr.position
    }

    pub fn expr(&self) -> &Expr {
        &self.expr.expr
    }
}

pub enum Statement {
    Expression(Expression),
}
