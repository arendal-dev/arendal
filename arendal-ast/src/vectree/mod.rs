use std::cmp::{Eq, PartialEq};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeRef {
    tree: i64,
    index: usize,
}

pub trait Node {
    fn get_children(&self, children: &mut Vec<NodeRef>);
}

struct InternalNode<T: Node> {
    node: T,
    parent: Option<NodeRef>,
}

type NodeVec<T> = Vec<InternalNode<T>>;

pub struct VecTree<T: Node> {
    nodes: NodeVec<T>,
}

pub enum Error {
    ChildNotFound(NodeRef),
    ChildHasParent(NodeRef),
}

pub struct Builder<T: Node> {
    tree: i64,
    nodes: NodeVec<T>,
    children: Vec<NodeRef>,
}

impl<T: Node> Builder<T> {
    pub fn add(&mut self, node: T) -> Result<NodeRef, Error> {
        self.children.clear();
        node.get_children(&mut self.children);
        // First pass: Validations
        for n in &self.children {
            if n.tree != self.tree {
                panic!("Invalid tree!")
            }
            let index = n.index;
            if index >= self.nodes.len() {
                return Err(Error::ChildNotFound(n.clone()));
            }
            let node = &self.nodes[index];
            if node.parent.is_some() {
                return Err(Error::ChildHasParent(n.clone()));
            }
        }
        let new_ref = NodeRef {
            tree: self.tree,
            index: self.nodes.len(),
        };
        // First pass: Update children
        for n in &mut self.children {
            let node = &mut self.nodes[n.index];
            node.parent = Some(new_ref.clone());
        }
        self.nodes.push(InternalNode { node, parent: None });
        Ok(new_ref)
    }
}

pub fn builder<T: Node>() -> Builder<T> {
    Builder {
        tree: rand::random(),
        nodes: Vec::default(),
        children: Vec::default(),
    }
}
