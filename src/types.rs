use vec1::Vec1;
pub type NodeIndex = usize;
pub type EdgeIndex = usize;

#[derive(Debug, Clone, PartialEq)]
pub struct NodeAddition {
    pub n: Node,
    pub e: EdgeSameSurface,
    pub next_id: usize,
    pub prev_id: usize
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: NodeIndex,
    pub x: f64,
    pub y: f64,
    pub inc: EdgeIndex,
    pub out: EdgeIndex,
    pub acrossness: Vec1<NodeIndex>,
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
    pub delta_x: f64,
    pub delta_y: f64
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
pub const OUTER: usize = 0;
pub const INNER: usize = 1;
#[derive(Debug, Clone)]
pub struct ThickSurface {
    pub layers: Vec<Graph>,
    pub edges: Vec<EdgeOuterToInner>,
}
