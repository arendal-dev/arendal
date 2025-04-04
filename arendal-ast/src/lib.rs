pub mod input;
pub mod position;
pub mod problem;
pub mod vectree;
pub mod keyword;
pub mod symbol;

use position::Position;
use problem::Result;
use symbol::{Symbol, TSymbol};
use vectree::{NodeId, VecTree};

pub enum NodeData {
    Expression(Expr),
}

pub struct Node {
    pub position: Position,
    pub data: NodeData,
}

impl vectree::Node for Node {
    fn get_children(&self, children: &mut Vec<NodeId>) {
        match &self.data {
            NodeData::Expression(e) => e.get_children(children),
        }
    }
}

pub struct AST {
    tree: VecTree<Node>,
}

pub struct Builder {
    builder: vectree::Builder<Node>,
}

impl Builder {
    pub fn add(&mut self, position: &Position, data: NodeData) -> Result<NodeId> {
        let node = Node {
            position: position.clone(),
            data,
        };
        match self.builder.add(node) {
            Ok(node) => problem::ok(node.clone()),
            Err(vectree::Error::ChildNotFound(_)) => {
                problem::error(position.clone(), "E0001", "ChildNotFound")
            }
            Err(vectree::Error::ChildHasParent(_)) => {
                problem::error(position.clone(), "E0002", "ChildHasParent")
            }
        }
    }
}

pub struct BuilderNoPosition {
    builder: Builder,
}

impl BuilderNoPosition {
    pub fn add(&mut self, data: NodeData) -> Result<NodeId> {
        self.builder.add(&Position::NoPosition, data)
    }
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
    pub expr: NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binary {
    pub op: BinaryOp,
    pub expr1: NodeId,
    pub expr2: NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seq {
    pub expr: NodeId,
    pub then: NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conditional {
    pub expr: NodeId,
    pub then: NodeId,
    pub otherwise: NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub exprs: Vec<NodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    LitInteger(i64),
    Symbol(Symbol),
    TSymbol(TSymbol),
    Unary(Unary),
    Binary(Binary),
    Block(Block),
    Conditional(Conditional),
    Seq(Seq),
}

impl Expr {
    fn get_children(&self, children: &mut Vec<NodeId>) {
        match self {
            Self::Unary(e) => children.push(e.expr.clone()),
            Self::Binary(e) => {
                children.push(e.expr1.clone());
                children.push(e.expr2.clone());
            }
            Self::Block(block) => {
                for e in &block.exprs {
                    children.push(e.clone());
                }
            }
            Self::Conditional(e) => {
                children.push(e.expr.clone());
                children.push(e.then.clone());
                children.push(e.otherwise.clone());
            }
            Self::Seq(e) => {
                children.push(e.expr.clone());
                children.push(e.then.clone());
            }
            _ => (),
        }
    }
}
