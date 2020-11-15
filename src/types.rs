pub type NodeIndex = usize;
pub type EdgeIndex = usize;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Acrossness{
    pub mid: Option<NodeIndex>,
    pub prev: Option<NodeIndex>,
    pub next: Option<NodeIndex>
}

#[derive(Clone, Copy, Debug)]
pub struct NodeAddition {
    pub prev_id: usize,
    pub next_id: usize,
    pub mid_acrossness: Acrossness,
    pub prev_acrossness: Acrossness,
    pub next_acrossness: Acrossness
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: NodeIndex,
    pub x: f64,
    pub y: f64,
    pub inc: EdgeIndex,
    pub out: EdgeIndex,
    pub acrossness: Acrossness, // Simple experimentation with prioritizing the push of across nodes. let's see
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
pub const OUTER: usize = 0;
pub const INNER: usize = 1;
#[derive(Debug, Clone)]
pub struct ThickSurface {
    pub layers: Vec<Graph>,
    pub edges: Vec<EdgeOuterToInner>,
}
