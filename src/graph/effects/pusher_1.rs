use graph::effects::apply_changes;
use graph::effects::helpers::*;
use graph::types::{Graph, Node, NodeChange, NodeChangeMap, ThickSurface, INNER, OUTER};
use graph::{closest_node_to_some_point, distance_between_nodes, distance_between_points};
use piston::input::keyboard::Key::Out;
use stitcher::types::Stitching;

fn inner_mods(modified_inners: &Vec<usize>, outer_changes: &NodeChangeMap, g: &Graph, ig: &Graph) -> NodeChangeMap {
    let mut ret = NodeChangeMap::new();
    for i in modified_inners {
        let three_closest = n_closest_outers(7, &ig.nodes[*i], outer_changes, g);
        let avg_change = avg_change_dumb(&ig.nodes[*i], &three_closest);
        ret.insert(*i, avg_change);
    }
    ret
}

pub fn push_inners(inner: &Graph, outer: &Graph, outer_changes: &NodeChangeMap, s: &Stitching) -> NodeChangeMap {
    let (most_outer, most_inner) = most_prev_next(outer_changes, outer);
    let (closest_inner_1, closest_inner_2) = closest_internal_nodes(most_outer, most_inner, inner);
    let modi = modified_inners(closest_inner_1, closest_inner_2, inner);
    inner_mods(&modi, outer_changes, outer, inner)
}
