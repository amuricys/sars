pub mod types;

use graph::{distance_between_nodes, distance_between_points};

use graph::types::{Node, ThickSurface, INNER, OUTER};
use std::collections::HashMap;
use vec1::Vec1;

fn greedy(ts: &ThickSurface) -> types::Stitching {
    let mut res = types::Stitching::new();

    let o = OUTER;
    let i = INNER;
    let (mut out_c, mut out_n) = (0, &ts.layers[o].nodes[0]);
    let (mut inn_c, mut inn_n) = (
        0,
        ts.layers[i]
            .nodes
            .iter()
            .min_by(|n1, n2| {
                distance_between_nodes(*n1, out_n)
                    .partial_cmp(&distance_between_nodes(*n2, out_n))
                    .unwrap()
            })
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

pub fn stitch(ts: &ThickSurface) -> types::Stitching {
    greedy(ts)
}
