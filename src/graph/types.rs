use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Smooth<L, R> {
    Count(L),
    Continuous(R),
}

impl Smooth<usize, f64> {
    pub fn as_f64(&self) -> f64 {
        match self {
            Smooth::Count(int) => *int as f64,
            Smooth::Continuous(flo) => *flo,
        }
    }

    pub fn add(self, rhs: f64) -> Smooth<usize, f64> {
        match self {
            Smooth::Count(int) => Smooth::Count(int + 1),
            Smooth::Continuous(flo) => Smooth::Continuous(flo + rhs),
        }
    }
}

pub type NodeIndex = usize;
#[derive(Debug)]
pub enum NodeChangeMap {
    NCM(HashMap<usize, NodeChange>),
}

impl NodeChangeMap {
    pub(crate) fn new() -> NodeChangeMap {
        NodeChangeMap::NCM(HashMap::new())
    }

    pub(crate) fn get(&self, k: &usize) -> Option<&NodeChange> {
        match self {
            NodeChangeMap::NCM(m) => m.get(k),
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
            NodeChangeMap::NCM(m) => match m.get_mut(&k) {
                Some(goddamn_thing) => {
                    let to_ins = NodeChange {
                        delta_x: (goddamn_thing.delta_x + v.delta_x) / 2.0,
                        delta_y: (goddamn_thing.delta_y + v.delta_y) / 2.0,
                        ..*goddamn_thing
                    };
                    m.insert(k, to_ins)
                }
                None => m.insert(k, v),
            },
        }
    }

    pub(crate) fn unwrap(&self) -> &HashMap<usize, NodeChange> {
        match self {
            NodeChangeMap::NCM(m) => m,
        }
    }
}

impl IntoIterator for NodeChangeMap {
    type Item = (usize, NodeChange);
    type IntoIter = std::collections::hash_map::IntoIter<usize, NodeChange>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            NodeChangeMap::NCM(m) => m.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a NodeChangeMap {
    type Item = (&'a usize, &'a NodeChange);
    type IntoIter = std::collections::hash_map::Iter<'a, usize, NodeChange>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            NodeChangeMap::NCM(m) => m.iter(),
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

    pub(crate) fn next_by<'a>(&'a self, g: &'a Graph, dist: usize) -> &'a Node {
        let mut n = self;
        for i in 0..dist {
            n = n.next(g);
        }
        n
    }

    pub(crate) fn prev<'a>(&self, g: &'a Graph) -> &'a Node {
        &g.nodes[self.prev_id]
    }

    pub(crate) fn pos(&self) -> (f64, f64) {
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

#[derive(Clone, Copy, Debug)]
pub struct NodeChange2 {
    pub id: NodeIndex,
    pub new_id: NodeIndex,
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

pub const OUTER: usize = 0;
pub const INNER: usize = 1;
#[derive(Debug, Clone)]
pub struct ThickSurface {
    pub layers: Vec<Graph>,
}

impl ThickSurface {
    pub(crate) fn new(outer: Graph, inner: Graph) -> ThickSurface {
        ThickSurface { layers: vec![outer, inner] }
    }
}
