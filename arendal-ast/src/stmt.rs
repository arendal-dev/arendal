use crate::{
    EMPTY, Empty,
    position::{EqNoPosition, Position},
    symbol::{Symbol, TSymbol},
};

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

pub type Binary = super::Binary<Empty>;
pub type Expr = super::Expr<Empty>;
pub type Expression = super::Expression<Empty>;

/*
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

impl Expr {
    pub fn to_expression(self, position: Position) -> Expression {
        Expression::new(position, self)
    }
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

    #[inline]
    pub fn to_statement(self) -> Statement {
        Statement::Expression(self)
    }
}

impl EqNoPosition for Expression {
    fn eq_nopos(&self, other: &Self) -> bool {
        match &self.expr.expr {
            Expr::Unary(u1) => match &other.expr.expr {
                Expr::Unary(u2) => u1.eq_nopos(u2),
                _ => false,
            },
            Expr::Binary(b1) => match &other.expr.expr {
                Expr::Binary(b2) => b1.eq_nopos(b2),
                _ => false,
            },
            Expr::Block(b1) => match &other.expr.expr {
                Expr::Block(b2) => b1.exprs.eq_nopos(&b2.exprs),
                _ => false,
            },
            Expr::Conditional(c1) => match &other.expr.expr {
                Expr::Conditional(c2) => c1.eq_nopos(c2),
                _ => false,
            },
            Expr::Seq(s1) => match &other.expr.expr {
                Expr::Seq(s2) => s1.eq_nopos(s2),
                _ => false,
            },
            e => e == &other.expr.expr,
        }
    }
}

*/

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
