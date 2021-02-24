use graph::{distance_between_nodes, distance_between_points};

use std::collections::HashMap;
use types::{Node, ThickSurface, INNER, OUTER};
use vec1::Vec1;

#[derive(Clone)]
pub enum ListMap {
    LMap(HashMap<usize, Vec1<(usize, f64, f64)>>),
}

impl ListMap {
    fn new() -> ListMap {
        ListMap::LMap(HashMap::new())
    }

    pub fn get(&self, key: usize) -> &Vec1<(usize, f64, f64)> {
        match self {
            ListMap::LMap(m) => match m.get(&key) {
                Some(v) => v,
                None => panic!("Why you do this? Lol don't get on a value that dont exist"),
            },
        }
    }

    pub fn put(&mut self, key: usize, val: (usize, f64, f64)) {
        match self {
            ListMap::LMap(m) => match m.get_mut(&key) {
                Some(v) => v.push(val),
                None => {
                    m.insert(key, Vec1::new(val));
                }
            },
        }
    }
}

impl IntoIterator for ListMap {
    type Item = (usize, vec1::Vec1<(usize, f64, f64)>);
    type IntoIter = std::collections::hash_map::IntoIter<usize, vec1::Vec1<(usize, f64, f64)>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ListMap::LMap(m) => m.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a ListMap {
    type Item = (&'a usize, &'a vec1::Vec1<(usize, f64, f64)>);
    type IntoIter = std::collections::hash_map::Iter<'a, usize, vec1::Vec1<(usize, f64, f64)>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ListMap::LMap(m) => m.iter(),
        }
    }
}

#[derive(Clone)]
pub enum Stitching {
    Stitch(Vec<ListMap>),
}

impl Stitching {
    fn new() -> Stitching {
        Stitching::Stitch(Vec::from([ListMap::new(), ListMap::new()]))
    }
    fn put(&mut self, inn: (usize, f64, f64), out: (usize, f64, f64)) {
        match self {
            Stitching::Stitch(layers) => {
                layers[OUTER].put(out.0, inn);
                layers[INNER].put(inn.0, out);
            }
        }
    }

    pub fn get(&self, layer_id: usize, n: Node) -> Vec1<usize> {
        match self {
            Stitching::Stitch(layers) => {
                match Vec1::try_from_vec(layers[layer_id].get(n.id).iter().map(|(id, _, _)| *id).collect::<Vec<usize>>()) {
                    Ok(s) => s,
                    Err(_)=> panic!("VA SE FUDER")
                }
            }
        }
    }

    pub fn get_closest_correspondent(&self, layer_id: usize, n: Node) -> usize {
        match self {
            Stitching::Stitch(layers) => {
                let corrs = layers[layer_id].get(n.id);
                corrs
                    .iter()
                    .min_by(|(_, x1, y1), (_, x2, y2)| {
                        distance_between_points(n.x, n.y, *x1, *y1)
                            .partial_cmp(&distance_between_points(n.x, n.y, *x2, *y2))
                            .unwrap()
                    })
                    .unwrap()
                    .0
            }
        }
    }
}

// fn put_and_walk(walking_ctr: &mut usize, res_stitching: &mut Stitching, inn_n: &Node, out_n: &Node, walking_node: &mut &Node, walking_graph: &Graph) {
//     *walking_ctr += 1;
//     res_stitching.put(inn_n.id, out_n.id);
//     *walking_node = walking_node.next(walking_graph);
// }

fn greedy(ts: &ThickSurface) -> Stitching {
    let mut res = Stitching::new();

    let o = OUTER;
    let i = INNER;
    let (mut out_c, mut out_n) = (0, &ts.layers[o].nodes[0]);
    let (mut inn_c, mut inn_n) = (
        0,
        ts.layers[i]
            .nodes
            .iter()
            .min_by(|n1, n2| distance_between_nodes(*n1, out_n).partial_cmp(&distance_between_nodes(*n2, out_n)).unwrap())
            .unwrap(),
    );
    while out_c <= ts.layers[o].nodes.len() && inn_c <= ts.layers[i].nodes.len() {
        if out_c >= ts.layers[o].nodes.len() {
            // put_and_walk(&mut inn_c, &mut res, &inn_n, &out_n, &mut inn_n, &ts.layers[i]);
            inn_c += 1;
            res.put((inn_n.id, inn_n.x, inn_n.y), (out_n.id, out_n.x, out_n.y));
            inn_n = inn_n.next(&ts.layers[i]);
        } else if inn_c >= ts.layers[i].nodes.len() {
            //put_and_walk(&mut out_c, &mut res, &inn_n, &out_n, &mut out_n, &ts.layers[o]);
            out_c += 1;
            res.put((inn_n.id, inn_n.x, inn_n.y), (out_n.id, out_n.x, out_n.y));
            out_n = out_n.next(&ts.layers[o]);
        } else {
            let dist_crossing_from_out = distance_between_nodes(out_n, inn_n.next(&ts.layers[i]));
            let dist_crossing_from_inn = distance_between_nodes(inn_n, out_n.next(&ts.layers[o]));
            if dist_crossing_from_inn < dist_crossing_from_out {
                //put_and_walk(&mut out_c, &mut res, &inn_n, &out_n, &mut out_n, &ts.layers[o]);
                out_c += 1;
                res.put((inn_n.id, inn_n.x, inn_n.y), (out_n.id, out_n.x, out_n.y));
                out_n = out_n.next(&ts.layers[o]);
            } else {
                //put_and_walk(&mut inn_c, &mut res, &inn_n, &out_n, &mut inn_n, &ts.layers[i]);
                inn_c += 1;
                res.put((inn_n.id, inn_n.x, inn_n.y), (out_n.id, out_n.x, out_n.y));
                inn_n = inn_n.next(&ts.layers[i]);
            }
        }
    }
    res
}

pub fn stitch(ts: &ThickSurface) -> Stitching {
    greedy(ts)
}
