pub mod parser;

use std::cmp::{Eq, PartialEq};
use std::collections::HashSet;
use std::sync::Arc;

use super::Integer;
use crate::error::{Error, Loc, Result, L};
use crate::symbol::{FQPath, Pkg, Symbol, TSymbol};
use crate::visibility::{Visibility, V};

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
pub struct Unary {
    pub op: UnaryOp,
    pub expr: ExprRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binary {
    pub op: BinaryOp,
    pub expr1: ExprRef,
    pub expr2: ExprRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seq {
    pub expr: ExprRef,
    pub then: ExprRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conditional {
    pub expr: ExprRef,
    pub then: ExprRef,
    pub otherwise: ExprRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub symbol: Symbol,
    pub expr: ExprRef,
}

pub type LAssignmentRef = Arc<L<Assignment>>;
pub type GAssignmentRef = Arc<L<V<Assignment>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub assignments: Vec<LAssignmentRef>,
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

pub type ExprRef = Arc<L<Expr>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewType {
    pub symbol: TSymbol,
    pub dfn: TypeDfn,
}

pub type NewTypeRef = Arc<L<V<NewType>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDfn {
    Singleton,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub path: FQPath,
    pub exprs: Vec<ExprRef>,
    pub assignments: Vec<GAssignmentRef>,
    pub types: Vec<NewTypeRef>,
}

pub type ModuleRef = Arc<Module>;

impl Module {
    fn new(path: FQPath) -> Self {
        Module {
            path,
            exprs: Vec::default(),
            assignments: Vec::default(),
            types: Vec::default(),
        }
    }
}

#[derive(Debug)]
pub struct Package {
    pub(crate) pkg: Pkg,
    pub(crate) modules: Vec<ModuleRef>,
}

impl Package {
    pub fn new(modules: Vec<ModuleRef>) -> Result<Self> {
        if modules.is_empty() {
            return Loc::None.err(Error::EmptyPackage);
        }
        let mut pkg: Option<Pkg> = None;
        let mut paths: HashSet<FQPath> = HashSet::default();
        for module in &modules {
            if !paths.insert(module.path.clone()) {
                return Loc::None.err(Error::DuplicateModule(module.path.clone()));
            }
            match &pkg {
                None => pkg = Some(module.path.pkg.clone()),
                Some(p) => {
                    if *p != module.path.pkg {
                        return Loc::None
                            .err(Error::UnexpectedPackage(p.clone(), module.path.pkg.clone()));
                    }
                }
            }
        }
        Ok(Self {
            pkg: pkg.unwrap(),
            modules,
        })
    }
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

    fn build(&self, expr: Expr) -> ExprRef {
        Arc::new(self.loc.wrap(expr))
    }

    pub fn lit_integer(&self, value: Integer) -> ExprRef {
        self.build(Expr::LitInteger(value))
    }

    pub fn symbol(&self, segments: Vec<Segment>, symbol: Symbol) -> ExprRef {
        self.build(Expr::Symbol(Q { segments, symbol }))
    }

    pub fn tsymbol(&self, segments: Vec<Segment>, symbol: TSymbol) -> ExprRef {
        self.build(Expr::TSymbol(Q { segments, symbol }))
    }

    pub fn unary(&self, op: UnaryOp, expr: ExprRef) -> ExprRef {
        self.build(Expr::Unary(Unary { op, expr }))
    }

    pub fn binary(&self, op: BinaryOp, expr1: ExprRef, expr2: ExprRef) -> ExprRef {
        self.build(Expr::Binary(Binary { op, expr1, expr2 }))
    }

    pub fn block(&self, mut assignments: Vec<LAssignmentRef>, mut exprs: Vec<ExprRef>) -> ExprRef {
        let na = assignments.len();
        let ne = exprs.len();
        assert!(na + ne > 0, "Blocks need to contain at least one statement");
        if na == 0 && ne == 1 {
            exprs.pop().unwrap()
        } else if na == 1 && ne == 0 {
            assignments.pop().unwrap().it.expr.clone()
        } else {
            self.build(Expr::Block(Block { assignments, exprs }))
        }
    }

    pub fn conditional(&self, expr: ExprRef, then: ExprRef, otherwise: ExprRef) -> ExprRef {
        self.build(Expr::Conditional(Conditional {
            expr,
            then,
            otherwise,
        }))
    }

    pub fn seq(&self, expr: ExprRef, then: ExprRef) -> ExprRef {
        self.build(Expr::Seq(Seq { expr, then }))
    }

    pub fn l_let(&self, symbol: Symbol, expr: ExprRef) -> LAssignmentRef {
        Arc::new(self.loc.wrap(Assignment { symbol, expr }))
    }

    pub fn g_let(&self, visibility: Visibility, symbol: Symbol, expr: ExprRef) -> GAssignmentRef {
        Arc::new(self.loc.wrap(visibility.wrap(Assignment { symbol, expr })))
    }

    fn new_type(&self, visibility: Visibility, symbol: TSymbol) -> NewTypeBuilder {
        NewTypeBuilder {
            loc: self.loc.clone(),
            visibility,
            symbol,
        }
    }
}

pub struct NewTypeBuilder {
    loc: Loc,
    visibility: Visibility,
    symbol: TSymbol,
}

impl NewTypeBuilder {
    fn build(self, dfn: TypeDfn) -> NewTypeRef {
        Arc::new(self.loc.to_wrap(self.visibility.wrap(NewType {
            symbol: self.symbol,
            dfn,
        })))
    }

    pub fn singleton(self) -> NewTypeRef {
        self.build(TypeDfn::Singleton)
    }
}
