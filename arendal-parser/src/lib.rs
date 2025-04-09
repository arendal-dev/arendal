mod lexer;

use lexer::{Lexeme, LexemeData, Lexemes};
use std::rc::Rc;

use ast::{
    common::{BinaryOp, UnaryOp},
    position::Position,
    problem::Problems,
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

pub type Unary = ast::common::Unary<ExprRef>;
pub type Binary = ast::common::Binary<ExprRef>;
pub type Seq = ast::common::Seq<ExprRef>;
pub type Conditional = ast::common::Conditional<ExprRef>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub exprs: Vec<ExprRef>,
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
pub struct ExprRef {
    expr: Rc<ExprData>,
}

impl ExprRef {
    pub fn position(&self) -> &Position {
        &self.expr.position
    }

    pub fn expr(&self) -> &Expr {
        &self.expr.expr
    }
}

pub enum Statement {
    Expression(ExprRef),
}

struct Parser {
    lexemes: Lexemes,
    index: usize,
    problems: Problems,
}

type EResult = std::result::Result<ExprRef, ()>;

impl Parser {
    fn peek(&self) -> Option<&Lexeme> {
        self.lexemes.get(self.index)
    }

    fn rule_primary(&self) -> EResult {
        if let Some(lexeme) = self.peek() {
            match &lexeme.data {
                LexemeData::Integer(n) => panic!("TODO: create expression"),
                _ => panic!("TODO: error"),
            }
        } else {
            panic!("TODO: error")
        }
    }
}
