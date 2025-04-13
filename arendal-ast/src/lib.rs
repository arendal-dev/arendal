pub mod common;
pub mod input;
pub mod keyword;
pub mod position;
pub mod problem;
pub mod stmt;
pub mod symbol;

use std::{fmt, rc::Rc};

use num::Integer;
use position::{EqNoPosition, Position};

pub trait NodeComponent: fmt::Debug + PartialEq + Eq {}

#[derive(Debug, PartialEq, Eq)]
struct Node<D: NodeComponent, T: NodeComponent> {
    pub position: Position,
    pub data: D,
    pub payload: T,
}

impl<D: NodeComponent + EqNoPosition, T: NodeComponent> EqNoPosition for Node<D, T> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.data.eq_nopos(&other.data) && self.payload == other.payload
    }
}

pub type Binary<T> = common::Binary<Expression<T>>;

#[derive(Debug, PartialEq, Eq)]
pub enum Expr<T: NodeComponent> {
    LitInteger(Integer),
    Binary(Binary<T>),
}

impl<T: NodeComponent> Expr<T> {
    pub fn to_expression(self, position: Position, payload: T) -> Expression<T> {
        Expression::new(position, self, payload)
    }
}

impl<T: NodeComponent> NodeComponent for Expr<T> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression<T: NodeComponent> {
    node: Rc<Node<Expr<T>, T>>,
}

impl<T: NodeComponent> Expression<T> {
    pub fn new(position: Position, data: Expr<T>, payload: T) -> Self {
        Self {
            node: Rc::new(Node {
                position,
                data,
                payload,
            }),
        }
    }

    pub fn position(&self) -> &Position {
        &self.node.position
    }

    pub fn expr(&self) -> &Expr<T> {
        &self.node.data
    }

    pub fn payload(&self) -> &T {
        &self.node.payload
    }
}

#[derive(Debug)]
pub struct AST<T: NodeComponent> {
    pub expression: Option<Expression<T>>,
}
