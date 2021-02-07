use std::collections::HashMap;
use std::cmp::Ord;
use types::{ThickSurface, Graph, Node, OUTER, INNER};
use graph::distance_between_nodes;
use vec1::Vec1;

pub enum ListMap {
    LMap(HashMap<usize, Vec1<usize>>)
}

impl ListMap {
    fn new() -> ListMap { ListMap::LMap(HashMap::new()) }

    fn get(&self, key: usize) -> &Vec1<usize> {
        match self {
            ListMap::LMap(m) => {
                match m.get(&key) {
                    Some(v) => v,
                    None => panic!("Why you do this? Lol don't get on a value that dont exist")
                }
            }
        }
    }

    fn put(&mut self, key: usize, val: usize) {
        match self {
            ListMap::LMap(m) => {
                match m.get_mut(&key) {
                    Some(v) => v.push( val),
                    None => { m.insert(key, Vec1::new(val)); }
                }
            }
        }
    }
}

impl IntoIterator for ListMap {
    type Item = (usize, vec1::Vec1<usize>);
    type IntoIter = std::collections::hash_map::IntoIter<usize, vec1::Vec1<usize>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ListMap::LMap(m) => {
                m.into_iter()
            }
        }
    }
}

impl<'a> IntoIterator for &'a ListMap {
    type Item = (&'a usize, &'a vec1::Vec1<usize>);
    type IntoIter = std::collections::hash_map::Iter<'a, usize, vec1::Vec1<usize>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ListMap::LMap(m) => {
                m.iter()
            }
        }
    }
}

pub enum Stitching {
    Stitch(Vec<ListMap>)
}

impl Stitching {
    fn new() -> Stitching { Stitching::Stitch(Vec::from([ListMap::new(), ListMap::new()])) }
    fn put(&mut self, inn: usize, out: usize) {
        match self {
            Stitching::Stitch(layers) => {
                layers[OUTER].put(out, inn);
                layers[INNER].put(inn, out);
            }
        }
    }
}

// fn put_and_walk(walking_ctr: &mut usize, res_stitching: &mut Stitching, inn_n: &Node, out_n: &Node, walking_node: &mut &Node, walking_graph: &Graph) {
//     *walking_ctr += 1;
//     res_stitching.put(inn_n.id, out_n.id);
//     *walking_node = walking_node.next(walking_graph);
// }


pub fn stitch(ts: &ThickSurface) -> Stitching {
    let mut res = Stitching::new();

    let o = OUTER;
    let i  = INNER;
    let (mut out_c, mut out_n) = (0, &ts.layers[o].nodes[0]);
    let (mut inn_c, mut inn_n) = (0, ts.layers[i].nodes.iter().min_by(
        |n1, n2| distance_between_nodes(*n1, out_n).partial_cmp(&distance_between_nodes(*n2, out_n)).unwrap()
    ).unwrap());
    while out_c <= ts.layers[o].nodes.len() && inn_c <= ts.layers[i].nodes.len() {
        if out_c >= ts.layers[o].nodes.len() {
           // put_and_walk(&mut inn_c, &mut res, &inn_n, &out_n, &mut inn_n, &ts.layers[i]);
            inn_c += 1;
            res.put(inn_n.id, out_n.id);
            inn_n = inn_n.next(&ts.layers[i]);
        } else if inn_c >= ts.layers[i].nodes.len() {
            //put_and_walk(&mut out_c, &mut res, &inn_n, &out_n, &mut out_n, &ts.layers[o]);
            out_c += 1;
            res.put(inn_n.id, out_n.id);
            out_n = out_n.next(&ts.layers[o]);
        } else {
            let dist_crossing_from_out = distance_between_nodes(out_n, inn_n.next(&ts.layers[i]));
            let dist_crossing_from_inn = distance_between_nodes(inn_n, out_n.next(&ts.layers[o]));
            if dist_crossing_from_inn < dist_crossing_from_out {
                //put_and_walk(&mut out_c, &mut res, &inn_n, &out_n, &mut out_n, &ts.layers[o]);
                out_c += 1;
                res.put(inn_n.id, out_n.id);
                out_n = out_n.next(&ts.layers[o]);
            } else {
                //put_and_walk(&mut inn_c, &mut res, &inn_n, &out_n, &mut inn_n, &ts.layers[i]);
                inn_c += 1;
                res.put(inn_n.id, out_n.id);
                inn_n = inn_n.next(&ts.layers[i]);
            }
        }
    }
    res
}