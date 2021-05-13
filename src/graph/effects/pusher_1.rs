use graph::types::{ThickSurface, NodeChangeMap, OUTER, NodeChange, Graph, Node, INNER};
use stitcher::types::Stitching;
use graph::effects::apply_changes;
use graph::{closest_node_to_some_point, distance_between_nodes, distance_between_points};
use piston::input::keyboard::Key::Out;
use graph::effects::helpers::*;


/*
This fn could have a few versions:
1. n_closest *of the changed nodes* PRE-change
2. n_closest *of changed nodes AND non-changed nodes* PRE-change
3. "    "     POST-change
4. "    "     "      "     POST-change
*/
fn n_closest_outers<'a>(n: usize, inner_node: &Node, outer_changes: &'a NodeChangeMap, g: &Graph) -> Vec<&'a NodeChange>{
    let mut ret = Vec::new();
    for (_, v) in outer_changes {
        ret.push(v);
    }
    ret.sort_by(|n1, n2| {
        distance_between_nodes(&g.nodes[n1.id], inner_node)
            .partial_cmp(&distance_between_nodes(&g.nodes[n2.id], inner_node))
            .unwrap()
    });
    let mut ret2 = Vec::new();
    for i in 0..n {
        ret2.push(ret[i]);
    }
    ret2
    //ret.iter().take(n).collect()
}

fn inner_mods(modified_inners: &Vec<usize>, outer_changes: &NodeChangeMap, g: &Graph, ig: &Graph) -> NodeChangeMap {
    let mut ret = NodeChangeMap::new();
    for i in modified_inners {
        let three_closest = n_closest_outers(7, &ig.nodes[*i], outer_changes, g);
        let avg_change = avg_change_dumb(&ig.nodes[*i], &three_closest);
        ret.insert(*i, avg_change);
    }
    ret
}

pub fn push_inners (inner: &Graph, outer: &Graph, outer_changes: &NodeChangeMap, s: &Stitching) -> NodeChangeMap {
    let (most_outer, most_inner) = most_prev_next(outer_changes, outer);
    let (closest_inner_1, closest_inner_2) = closest_internal_nodes(most_outer, most_inner, inner);
    let modi = modified_inners(closest_inner_1, closest_inner_2, inner);
    inner_mods(&modi, outer_changes, outer, inner)
}