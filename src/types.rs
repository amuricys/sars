use vec1::Vec1;
use std::collections::HashMap;

pub type NodeIndex = usize;
pub type EdgeIndex = usize;

#[derive(Debug, Clone, PartialEq)]
pub struct NodeAddition {
    pub n: Node
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: NodeIndex,
    pub x: f64,
    pub y: f64,
    pub next_id: NodeIndex,
    pub prev_id: NodeIndex,
    pub acrossness: Vec1<NodeIndex>,
}

impl Node {
    pub(crate) fn next<'a>(&self, g: &'a Graph) -> &'a Node {
        &g.nodes[self.next_id]
    }

    pub(crate) fn prev<'a>(&self, g: &'a Graph) -> &'a Node {
        &g.nodes[self.prev_id]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NodeChange {
    pub id: NodeIndex,
    pub cur_x: f64,
    pub cur_y: f64,
    pub delta_x: f64,
    pub delta_y: f64
}

#[derive(Debug, Clone)]
pub struct Graph {
    pub nodes: Vec<Node>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct EdgeOuterToInner {
    pub target: NodeIndex,
    pub source: NodeIndex,
}
pub const OUTER: usize = 0;
pub const INNER: usize = 1;
#[derive(Debug, Clone)]
pub struct ThickSurface {
    pub layers: Vec<Graph>,
    pub edges: Vec<EdgeOuterToInner>,
}
