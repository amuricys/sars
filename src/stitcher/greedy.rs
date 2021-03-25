use graph::{distance_between_nodes, distance_between_points};

use graph::types::{Graph, Node};
use std::collections::HashMap;
use vec1::Vec1;

use stitcher::types;

pub fn stitch(outer: &Graph, inner: &Graph) -> types::Stitching {
    let mut res = types::Stitching::new();

    let (mut out_c, mut out_n) = (0, &outer.nodes[0]);
    let (mut inn_c, mut inn_n) = (
        0,
        inner
            .nodes
            .iter()
            .min_by(|n1, n2| {
                distance_between_nodes(*n1, out_n)
                    .partial_cmp(&distance_between_nodes(*n2, out_n))
                    .unwrap()
            })
            .unwrap(),
    );
    while out_c <= outer.nodes.len() && inn_c <= inner.nodes.len() {
        if out_c >= outer.nodes.len() {
            // put_and_walk(&mut inn_c, &mut res, &inn_n, &out_n, &mut inn_n, &inner);
            inn_c += 1;
            res.put((inn_n.id, inn_n.x, inn_n.y), (out_n.id, out_n.x, out_n.y));
            inn_n = inn_n.next(&inner);
        } else if inn_c >= inner.nodes.len() {
            //put_and_walk(&mut out_c, &mut res, &inn_n, &out_n, &mut out_n, &outer);
            out_c += 1;
            res.put((inn_n.id, inn_n.x, inn_n.y), (out_n.id, out_n.x, out_n.y));
            out_n = out_n.next(&outer);
        } else {
            let dist_crossing_from_out = distance_between_nodes(out_n, inn_n.next(&inner));
            let dist_crossing_from_inn = distance_between_nodes(inn_n, out_n.next(&outer));
            if dist_crossing_from_inn < dist_crossing_from_out {
                //put_and_walk(&mut out_c, &mut res, &inn_n, &out_n, &mut out_n, &outer);
                out_c += 1;
                res.put((inn_n.id, inn_n.x, inn_n.y), (out_n.id, out_n.x, out_n.y));
                out_n = out_n.next(&outer);
            } else {
                //put_and_walk(&mut inn_c, &mut res, &inn_n, &out_n, &mut inn_n, &inner);
                inn_c += 1;
                res.put((inn_n.id, inn_n.x, inn_n.y), (out_n.id, out_n.x, out_n.y));
                inn_n = inn_n.next(&inner);
            }
        }
    }
    res
}
