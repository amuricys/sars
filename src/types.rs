
use std::collections::HashMap;


pub type NodeIndex = usize;
pub type EdgeIndex = usize;
pub enum NodeChangeMap {
    FuckYes(HashMap<usize, NodeChange>),
}

impl NodeChangeMap {
    pub(crate) fn new() -> NodeChangeMap {
        NodeChangeMap::FuckYes(HashMap::new())
    }

    pub(crate) fn get(&self, k: &usize) -> Option<&NodeChange> {
        match self {
            NodeChangeMap::FuckYes(m) => m.get(k),
        }
    }
    /* This insertion mechanism implements a moving average on every insertion to a map of NodeChanges, instead of overriding
       Meaning that if you have a map like this:
       {1 -> NodeChange{delta_x: 1.0, delta_y: 2.0}}
       and you insert the keyvalue pair (1, NodeChange{delta_x: 3.0, delta_y: -2.0}), the map will become
       {1 -> NodeChange{delta_x: 1.0 + 3.0 / 2, delta_y: 2.0 - 2.0 / 2} = {1 -> NodeChange{delta_x: 2.0, delta_y: 0.0}}}
    */
    pub(crate) fn insert(&mut self, k: usize, v: NodeChange) -> Option<NodeChange> {
        match self {
            NodeChangeMap::FuckYes(m) => match m.get(&k) {
                Some(goddamn_thing) => m.insert(
                    k,
                    NodeChange {
                        delta_x: (goddamn_thing.delta_x + v.delta_x) / 2.0,
                        delta_y: (goddamn_thing.delta_y + v.delta_y) / 2.0,
                        ..*goddamn_thing
                    },
                ),
                None => m.insert(k, v),
            },
        }
    }

    pub(crate) fn unwrap(&self) -> &HashMap<usize, NodeChange> {
        match self {
            NodeChangeMap::FuckYes(m) => m,
        }
    }
}

impl IntoIterator for NodeChangeMap {
    type Item = (usize, NodeChange);
    type IntoIter = std::collections::hash_map::IntoIter<usize, NodeChange>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            NodeChangeMap::FuckYes(m) => m.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a NodeChangeMap {
    type Item = (&'a usize, &'a NodeChange);
    type IntoIter = std::collections::hash_map::Iter<'a, usize, NodeChange>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            NodeChangeMap::FuckYes(m) => m.iter(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeAddition {
    pub n: Node,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: NodeIndex,
    pub x: f64,
    pub y: f64,
    pub next_id: NodeIndex,
    pub prev_id: NodeIndex,
}

impl Node {
    pub(crate) fn next<'a>(&self, g: &'a Graph) -> &'a Node {
        &g.nodes[self.next_id]
    }

    pub(crate) fn prev<'a>(&self, g: &'a Graph) -> &'a Node {
        &g.nodes[self.prev_id]
    }

    pub(crate) fn pos<'a>(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NodeChange {
    pub id: NodeIndex,
    pub cur_x: f64,
    pub cur_y: f64,
    pub delta_x: f64,
    pub delta_y: f64,
}

impl NodeChange {
    pub(crate) fn changed_pos(&self) -> (f64, f64) {
        (self.cur_x + self.delta_x, self.cur_y + self.delta_y)
    }
}

#[derive(Debug, Clone)]
pub struct Graph {
    pub nodes: Vec<Node>,
}

impl Graph {
    pub fn next(&self, id: usize) -> &Node {
        self.nodes[id].next(self)
    }
    pub fn prev(&self, id: usize) -> &Node {
        self.nodes[id].prev(self)
    }
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
    pub softness_factor: f64, // <- how much should closeness of nodes in different surfaces impact pushes?
    pub how_smooth: usize,
    pub node_addition_threshold: f64,
    pub node_deletion_threshold: f64,
    pub low_high: (f64, f64),
    pub recorders: Vec<String>,
}
