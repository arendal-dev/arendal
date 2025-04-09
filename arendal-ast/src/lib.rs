pub mod common;
pub mod input;
pub mod keyword;
pub mod position;
pub mod problem;
pub mod stmt;
pub mod symbol;

use position::Position;

pub enum NodeData {
    Expression,
}

pub struct Node {
    pub position: Position,
    pub data: NodeData,
}

pub struct AST {}
