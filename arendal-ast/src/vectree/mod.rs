use std::cmp::{Eq, PartialEq};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeId {
    tree_id: i64,
    index: usize,
}

pub trait Node {
    fn get_children(&self, children: &mut Vec<NodeId>);
}

struct InternalNode<T: Node> {
    id: NodeId,
    node: T,
    parent: Option<NodeId>,
}

type NodeVec<T> = Vec<InternalNode<T>>;

pub struct VecTree<T: Node> {
    tree_id: i64,
    nodes: NodeVec<T>,
    top_level: Vec<NodeId>,
}

impl<T: Node> VecTree<T> {
    pub fn get(&self, id: &NodeId) -> Option<&T> {
        if self.tree_id != id.tree_id {
            panic!("Wrong tree")
        }
        self.nodes.get(id.index).map(|n| &n.node)
    }

    pub fn top_level_iter(&self) -> TopLevelIter<'_, T> {
        TopLevelIter {
            tree: self,
            iter: self.top_level.iter(),
        }
    }
}

pub struct TopLevelIter<'a, T: Node> {
    tree: &'a VecTree<T>,
    iter: std::slice::Iter<'a, NodeId>,
}

impl<'a, T: Node> Iterator for TopLevelIter<'a, T> {
    type Item = (NodeId, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|id| (id.clone(), &self.tree.nodes[id.index].node))
    }
}

pub enum Error {
    ChildNotFound(NodeId),
    ChildHasParent(NodeId),
}

pub struct Builder<T: Node> {
    tree_id: i64,
    nodes: NodeVec<T>,
    children: Vec<NodeId>,
}

impl<T: Node> Builder<T> {
    pub fn add(&mut self, node: T) -> Result<NodeId, Error> {
        self.children.clear();
        node.get_children(&mut self.children);
        // First pass: Validations
        for n in &self.children {
            if n.tree_id != self.tree_id {
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
        let id = NodeId {
            tree_id: self.tree_id,
            index: self.nodes.len(),
        };
        // First pass: Update children
        for n in &mut self.children {
            let node = &mut self.nodes[n.index];
            node.parent = Some(id.clone());
        }
        self.nodes.push(InternalNode {
            id: id.clone(),
            node,
            parent: None,
        });
        Ok(id)
    }

    pub fn build(self) -> VecTree<T> {
        let mut top_level = Vec::<NodeId>::default();
        for node in &self.nodes {
            if node.parent.is_none() {
                top_level.push(node.id.clone());
            }
        }
        VecTree {
            tree_id: self.tree_id,
            nodes: self.nodes,
            top_level,
        }
    }
}

pub fn builder<T: Node>() -> Builder<T> {
    Builder {
        tree_id: rand::random(),
        nodes: Vec::default(),
        children: Vec::default(),
    }
}
