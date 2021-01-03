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
        if let Some(x) = g.nodes.get(&self.next_id) {
            x
        } else {
            panic!(format!("node {:?}'s next_id is not in {:?}", self, g))

        }
    }

    pub(crate) fn prev<'a>(&self, g: &'a Graph) -> &'a Node {
        if let Some(x) = g.nodes.get(&self.prev_id) {
            x
        } else {
            panic!(format!("node {:?}'s prev_id is not in {:?}", self, g))
        }

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
    pub nodes: HashMap<usize, Node>,
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
}

pub struct Params {
    pub initial_thickness: f64,
    pub initial_radius: f64,
    pub initial_num_points: usize,
    pub initial_temperature: f64,
    pub initial_gray_matter_area: f64,
    pub compression_factor: f64,
    pub softness_factor: f64,    // <- how much should closeness of nodes in different surfaces impact pushes?
    pub how_smooth: usize,
    pub node_addition_threshold: f64,
    pub node_deletion_threshold: f64,
    pub low_high: (f64, f64),
    pub recorders: Vec<String>
}

