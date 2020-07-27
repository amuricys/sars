use std::collections::HashSet;

pub type NodeIndex = usize;
pub type EdgeIndex = usize;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: NodeIndex,
    pub x: f64,
    pub y: f64,
    pub inc: EdgeIndex,
    pub out: EdgeIndex,
    pub across: HashSet<NodeIndex>, // TODO: Inter-layer stuff is hard
}

impl Node {
    pub(crate) fn next<'a>(&self, g: &'a Graph) -> &'a Node {
        &g.nodes[g.edges[self.out].target]
    }

    pub(crate) fn prev<'a>(&self, g: &'a Graph) -> &'a Node {
        &g.nodes[g.edges[self.inc].source]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NodeChange {
    pub id: NodeIndex,
    pub cur_x: f64,
    pub cur_y: f64,
    pub new_x: f64,
    pub new_y: f64,

}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct EdgeSameSurface {
    pub id: EdgeIndex,
    pub target: NodeIndex,
    pub source: NodeIndex,
}

#[derive(Debug, Clone)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<EdgeSameSurface>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct EdgeOuterToInner {
    pub target: NodeIndex,
    pub source: NodeIndex,
}
#[derive(Debug, Clone)]
pub struct ThickSurface {
    pub inner: Graph,
    pub outer: Graph,
    pub edges: Vec<EdgeOuterToInner>,
}
