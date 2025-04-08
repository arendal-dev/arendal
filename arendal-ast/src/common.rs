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
pub struct Unary<E> {
    pub op: UnaryOp,
    pub expr: E,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binary<E> {
    pub op: BinaryOp,
    pub expr1: E,
    pub expr2: E,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seq<E> {
    pub expr: E,
    pub then: E,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conditional<E> {
    pub expr: E,
    pub then: E,
    pub otherwise: E,
}
