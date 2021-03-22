use graph::{distance_between_nodes, distance_between_points};

use graph::types::{Node, ThickSurface, INNER, OUTER};
use std::collections::HashMap;
use vec1::Vec1;

#[derive(Clone, Debug)]
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
                Some(v) => {
                    if !v.iter().any(|(x, _, _)| *x == val.0) {
                        // No duplicates in Stitching correspondences
                        v.push(val)
                    }
                }
                None => {
                    m.insert(key, Vec1::new(val));
                }
            },
        }
    }

    pub fn len(&self) -> usize {
        match self {
            ListMap::LMap(m) => m.len(),
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

#[derive(Clone, Debug)]
pub enum Stitching {
    Stitch(Vec<ListMap>),
}

impl Stitching {
    pub fn new() -> Stitching {
        Stitching::Stitch(Vec::from([ListMap::new(), ListMap::new()]))
    }
    pub fn put(&mut self, inn: (usize, f64, f64), out: (usize, f64, f64)) {
        match self {
            Stitching::Stitch(layers) => {
                layers[OUTER].put(out.0, inn);
                layers[INNER].put(inn.0, out);
            }
        }
    }

    pub fn get(&self, layer_id: usize, n: &Node) -> Vec1<usize> {
        match self {
            Stitching::Stitch(layers) => match Vec1::try_from_vec(layers[layer_id].get(n.id).iter().map(|(id, _, _)| *id).collect::<Vec<usize>>()) {
                Ok(s) => s,
                Err(_) => panic!("VA SE FUDER"),
            },
        }
    }

    pub fn get_closest_correspondent(&self, layer_id: usize, n: &Node) -> usize {
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

    pub fn len(&self) -> usize {
        match self {
            Stitching::Stitch(layers) => {
                fn corrs_amt(v: &ListMap) -> usize {
                    let mut amt = 0;
                    for (_, c) in v {
                        amt = amt + c.len();
                    }
                    amt
                }
                let (outer_amt, inner_amt) = (corrs_amt(&layers[OUTER]), corrs_amt(&layers[INNER]));
                if outer_amt == inner_amt {
                    outer_amt
                } else {
                    panic!(format!("Outer: {:?}; Inner: {:?}. Should never happen.", outer_amt, inner_amt))
                }
            }
        }
    }
}
