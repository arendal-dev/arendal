pub mod parser;

use std::cmp::{Eq, PartialEq};

use im::HashMap;

use super::Integer;
use crate::error::{Loc, L};
use crate::symbol::{Path, Pkg, Symbol, TSymbol};
use crate::visibility::V;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: L<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub expr1: L<Expr>,
    pub expr2: L<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seq {
    pub expr: L<Expr>,
    pub then: L<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conditional {
    pub expr: L<Expr>,
    pub then: L<Expr>,
    pub otherwise: L<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub symbol: Symbol,
    pub expr: L<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub assignments: Vec<L<Assignment>>,
    pub exprs: Vec<L<Expr>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    LitInteger(Integer),
    Symbol(Q<Symbol>),
    TSymbol(Q<TSymbol>),
    Unary(Box<UnaryExpr>),
    Binary(Box<BinaryExpr>),
    Block(Box<Block>),
    Conditional(Box<Conditional>),
    Seq(Box<Seq>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewType {
    pub symbol: TSymbol,
    pub dfn: TypeDfn,
}

pub type LNewType = L<NewType>;
pub type LVNewType = L<V<NewType>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDfn {
    Singleton,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Module {
    pub exprs: Vec<L<Expr>>,
    pub assignments: Vec<L<V<Assignment>>>,
    pub types: Vec<LVNewType>,
}

#[derive(Debug)]
pub struct Package {
    pub(crate) pkg: Pkg,
    pub(crate) modules: HashMap<Path, Module>,
}

impl Package {
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }
}

pub struct Builder {
    loc: Loc,
}

impl Builder {
    pub const fn new(loc: Loc) -> Self {
        Builder { loc }
    }

    pub const fn none() -> Self {
        Self::new(Loc::None)
    }

    fn build(&self, expr: Expr) -> L<Expr> {
        self.loc.wrap(expr)
    }

    pub fn lit_integer(&self, value: Integer) -> L<Expr> {
        self.build(Expr::LitInteger(value))
    }

    pub fn symbol(&self, segments: Vec<Segment>, symbol: Symbol) -> L<Expr> {
        self.build(Expr::Symbol(Q { segments, symbol }))
    }

    pub fn tsymbol(&self, segments: Vec<Segment>, symbol: TSymbol) -> L<Expr> {
        self.build(Expr::TSymbol(Q { segments, symbol }))
    }

    pub fn unary(&self, op: UnaryOp, expr: L<Expr>) -> L<Expr> {
        self.build(Expr::Unary(Box::new(UnaryExpr { op, expr })))
    }

    pub fn binary(&self, op: BinaryOp, expr1: L<Expr>, expr2: L<Expr>) -> L<Expr> {
        self.build(Expr::Binary(Box::new(BinaryExpr { op, expr1, expr2 })))
    }

    pub fn block(&self, mut assignments: Vec<L<Assignment>>, mut exprs: Vec<L<Expr>>) -> L<Expr> {
        let na = assignments.len();
        let ne = exprs.len();
        assert!(na + ne > 0, "Blocks need to contain at least one statement");
        if na == 0 && ne == 1 {
            exprs.pop().unwrap()
        } else if na == 1 && ne == 0 {
            assignments.pop().unwrap().it.expr
        } else {
            self.build(Expr::Block(Box::new(Block { assignments, exprs })))
        }
    }

    pub fn conditional(&self, expr: L<Expr>, then: L<Expr>, otherwise: L<Expr>) -> L<Expr> {
        self.build(Expr::Conditional(Box::new(Conditional {
            expr,
            then,
            otherwise,
        })))
    }

    pub fn seq(&self, expr: L<Expr>, then: L<Expr>) -> L<Expr> {
        self.build(Expr::Seq(Box::new(Seq { expr, then })))
    }

    pub fn assignment(&self, symbol: Symbol, expr: L<Expr>) -> L<Assignment> {
        self.loc.wrap(Assignment { symbol, expr })
    }

    fn new_type(&self, symbol: TSymbol) -> NewTypeBuilder {
        NewTypeBuilder {
            loc: self.loc.clone(),
            symbol,
        }
    }
}

pub struct NewTypeBuilder {
    loc: Loc,
    symbol: TSymbol,
}

impl NewTypeBuilder {
    fn build(self, dfn: TypeDfn) -> LNewType {
        self.loc.to_wrap(NewType {
            symbol: self.symbol,
            dfn,
        })
    }

    pub fn singleton(self) -> LNewType {
        self.build(TypeDfn::Singleton)
    }
}
