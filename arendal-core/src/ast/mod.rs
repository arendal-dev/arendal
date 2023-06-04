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

impl L<Assignment> {
    pub(crate) fn to_bstmt(self) -> BStmt {
        BStmt::Assignment(Box::new(self))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    LitInteger(Integer),
    Symbol(Q<Symbol>),
    TSymbol(Q<TSymbol>),
    Unary(Box<UnaryExpr>),
    Binary(Box<BinaryExpr>),
    Block(Vec<BStmt>),
    Conditional(Box<Conditional>),
}

impl L<Expr> {
    pub(crate) fn to_bstmt(self) -> BStmt {
        BStmt::Expr(Box::new(self))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BStmt {
    Assignment(Box<L<Assignment>>),
    Expr(Box<L<Expr>>),
}

pub struct ExprBuilder {
    loc: Loc,
}

impl ExprBuilder {
    pub const fn new(loc: Loc) -> Self {
        ExprBuilder { loc }
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

    pub fn block(&self, stmts: Vec<BStmt>) -> L<Expr> {
        assert!(
            !stmts.is_empty(),
            "Blocks need to contain at least one statement"
        );
        if stmts.len() == 1 {
            if let BStmt::Expr(e) = &stmts[0] {
                return *e.clone();
            }
        }
        self.build(Expr::Block(stmts))
    }

    pub fn conditional(&self, expr: L<Expr>, then: L<Expr>, otherwise: L<Expr>) -> L<Expr> {
        self.build(Expr::Conditional(Box::new(Conditional {
            expr,
            then,
            otherwise,
        })))
    }

    pub fn assignment(&self, symbol: Symbol, expr: L<Expr>) -> L<Assignment> {
        self.loc.wrap(Assignment { symbol, expr })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDefinition {
    pub loc: Loc,
    pub symbol: TSymbol,
    pub dfn: TypeDfn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDfn {
    Singleton,
}

pub struct TypeDfnBuilder {
    loc: Loc,
    symbol: TSymbol,
}

impl TypeDfnBuilder {
    pub const fn new(loc: Loc, symbol: TSymbol) -> Self {
        TypeDfnBuilder { loc, symbol }
    }

    fn build(self, dfn: TypeDfn) -> TypeDefinition {
        TypeDefinition {
            loc: self.loc,
            symbol: self.symbol,
            dfn,
        }
    }

    pub fn singleton(self) -> TypeDefinition {
        self.build(TypeDfn::Singleton)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Module {
    pub exprs: Vec<L<Expr>>,
    pub assignments: Vec<L<V<Assignment>>>,
    pub types: Vec<TypeDefinition>,
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
