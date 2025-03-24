use std::cmp::{Eq, PartialEq};

use super::Integer;
use crate::problem::{error, ok, NoPosition, Position, Result};
use crate::symbol::{FQPath, Pkg, Symbol, TSymbol};
use crate::vectree;

pub use vectree::NodeRef;

pub enum NodeData {
    Expression(Expr),
}

pub struct Node<P: Position> {
    pub position: P,
    pub data: NodeData,
}

impl<P: Position> vectree::Node for Node<P> {
    fn get_children(&self, children: &mut Vec<NodeRef>) {
        match &self.data {
            NodeData::Expression(e) => e.get_children(children),
        }
    }
}

pub struct AST<P: Position> {
    tree: vectree::VecTree<Node<P>>,
}

pub struct Builder<P: Position> {
    builder: vectree::Builder<Node<P>>,
}

impl<P: Position> Builder<P> {
    pub fn add(&mut self, position: &P, data: NodeData) -> Result<P, NodeRef> {
        let node = Node {
            position: position.clone(),
            data,
        };
        match self.builder.add(node) {
            Ok(node) => ok(node.clone()),
            Err(vectree::Error::ChildNotFound(_)) => {
                error(position.clone(), "E0001".into(), "ChildNotFound".into())
            }
            Err(vectree::Error::ChildHasParent(_)) => {
                error(position.clone(), "E0002".into(), "ChildHasParent".into())
            }
        }
    }
}

pub struct BuilderNoPosition {
    builder: Builder<NoPosition>,
}

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
    pub expr: NodeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binary {
    pub op: BinaryOp,
    pub expr1: NodeRef,
    pub expr2: NodeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seq {
    pub expr: NodeRef,
    pub then: NodeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conditional {
    pub expr: NodeRef,
    pub then: NodeRef,
    pub otherwise: NodeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub exprs: Vec<NodeRef>,
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
    fn get_children(&self, children: &mut Vec<NodeRef>) {
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
