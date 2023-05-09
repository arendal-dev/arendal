pub mod parser;

use std::cmp::{Eq, PartialEq};
use std::fmt;
use std::fmt::Debug;
use std::slice::Iter;

use im::HashMap;

use super::Integer;
use crate::error::Loc;
use crate::symbol::{Symbol, TSymbol};

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

#[derive(PartialEq, Eq)]
pub struct Expression {
    pub loc: Loc,
    pub expr: Expr,
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.expr.fmt(f)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Expression,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub expr1: Expression,
    pub expr2: Expression,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Conditional {
    pub expr: Expression,
    pub then: Expression,
    pub otherwise: Expression,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AssignmentExpr {
    pub symbol: Symbol,
    pub expr: Expression,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    LitInteger(Integer),
    Symbol(Symbol),
    TSymbol(TSymbol),
    Unary(Box<UnaryExpr>),
    Binary(Box<BinaryExpr>),
    Block(Vec<Expression>),
    Conditional(Box<Conditional>),
    Assignment(Box<AssignmentExpr>),
}

pub struct ExprBuilder {
    loc: Loc,
}

impl ExprBuilder {
    pub const fn new(loc: Loc) -> Self {
        ExprBuilder { loc }
    }

    pub const fn none() -> Self {
        Self::new(Loc::none())
    }

    fn build(&self, expr: Expr) -> Expression {
        Expression {
            loc: self.loc.clone(),
            expr,
        }
    }

    pub fn lit_integer(&self, value: Integer) -> Expression {
        self.build(Expr::LitInteger(value))
    }

    pub fn symbol(&self, symbol: Symbol) -> Expression {
        self.build(Expr::Symbol(symbol))
    }

    pub fn tsymbol(&self, symbol: TSymbol) -> Expression {
        self.build(Expr::TSymbol(symbol))
    }

    pub fn unary(&self, op: UnaryOp, expr: Expression) -> Expression {
        self.build(Expr::Unary(Box::new(UnaryExpr { op, expr })))
    }

    pub fn binary(&self, op: BinaryOp, expr1: Expression, expr2: Expression) -> Expression {
        self.build(Expr::Binary(Box::new(BinaryExpr { op, expr1, expr2 })))
    }

    pub fn block(&self, mut exprs: Vec<Expression>) -> Expression {
        assert!(
            !exprs.is_empty(),
            "Blocks need to contain at least one expression"
        );
        if exprs.len() == 1 {
            exprs.pop().unwrap()
        } else {
            self.build(Expr::Block(exprs))
        }
    }

    pub fn conditional(
        &self,
        expr: Expression,
        then: Expression,
        otherwise: Expression,
    ) -> Expression {
        self.build(Expr::Conditional(Box::new(Conditional {
            expr,
            then,
            otherwise,
        })))
    }

    pub fn assignment(&self, symbol: Symbol, expr: Expression) -> Expression {
        self.build(Expr::Assignment(Box::new(AssignmentExpr { symbol, expr })))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TypeDefinition {
    pub loc: Loc,
    pub symbol: TSymbol,
    pub dfn: TypeDfn,
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub enum ModuleItem {
    Expression(Expression),
    TypeDefinition(TypeDefinition),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    items: Vec<ModuleItem>,
}

impl Module {
    pub fn new(items: Vec<ModuleItem>) -> Self {
        Self { items }
    }

    pub fn iter(&self) -> Iter<'_, ModuleItem> {
        self.items.iter()
    }
}

impl<'a> IntoIterator for &'a Module {
    type Item = &'a ModuleItem;
    type IntoIter = Iter<'a, ModuleItem>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub type Path = Vec<Symbol>;

#[derive(Debug)]
pub struct Package {
    modules: HashMap<Path, Module>,
}

impl Package {
    fn new() -> Self {
        Package {
            modules: Default::default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    pub fn iter(&self) -> im::hashmap::Iter<'_, Path, Module> {
        self.modules.iter()
    }
}

impl<'a> IntoIterator for &'a Package {
    type Item = (&'a Path, &'a Module);
    type IntoIter = im::hashmap::Iter<'a, Path, Module>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
