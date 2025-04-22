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

pub trait Payload: fmt::Debug + PartialEq + Eq {}

trait Data: fmt::Debug + EqNoPosition {}

#[derive(Debug, PartialEq, Eq)]
struct Node<D: Data, P: Payload> {
    pub position: Position,
    pub data: D,
    pub payload: P,
}

impl<D: Data, P: Payload> EqNoPosition for Node<D, P> {
    fn eq_nopos(&self, other: &Self) -> bool {
        self.data.eq_nopos(&other.data) && self.payload == other.payload
    }
}

pub type Binary<T> = common::Binary<Expression<T>>;

#[derive(Debug, PartialEq, Eq)]
pub enum Expr<P: Payload> {
    LitInteger(Integer),
    Binary(Binary<P>),
}

impl<P: Payload> Expr<P> {
    pub fn to_expression(self, position: Position, payload: P) -> Expression<P> {
        Expression::new(position, self, payload)
    }
}

impl<P: Payload> Data for Expr<P> {}

impl<P: Payload> EqNoPosition for Expr<P> {
    fn eq_nopos(&self, other: &Self) -> bool {
        false // TODO
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression<T: Payload> {
    node: Rc<Node<Expr<T>, T>>,
}

impl<T: Payload> Expression<T> {
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
pub struct AST<T: Payload> {
    pub expression: Option<Expression<T>>,
}
