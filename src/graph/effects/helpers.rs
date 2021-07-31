use graph::types::{Graph, Node, NodeChange, NodeChangeMap};
use graph::{closest_node_to_some_point, distance_between_nodes};

pub(crate) fn most_prev_next<'a>(ncm: &NodeChangeMap, g: &'a Graph) -> (&'a Node, &'a Node) {
    let (_, most_next) = ncm
        .unwrap()
        .iter()
        .find(|(_, v)| match ncm.get(&g.next(v.id).id) {
            None => true,
            _ => false,
        })
        .unwrap();
    let (_, most_prev) = ncm
        .unwrap()
        .iter()
        .find(|(_, v)| match ncm.get(&g.prev(v.id).id) {
            None => true,
            _ => false,
        })
        .unwrap();
    (&g.nodes[most_prev.id], &g.nodes[most_next.id])
}

pub(crate) fn closest_internal_nodes<'a>(most_outer_prev: &Node, most_outer_next: &Node, ig: &'a Graph) -> (&'a Node, &'a Node) {
    (
        closest_node_to_some_point(ig, most_outer_prev.x, most_outer_prev.y),
        closest_node_to_some_point(ig, most_outer_next.x, most_outer_next.y),
    )
}

pub(crate) fn modified_inners(closest_inner_1: &Node, closest_inner_2: &Node, ig: &Graph) -> Vec<usize> {
    let (mut clkwise, mut ctr_clkwise) = (Vec::new(), Vec::new());
    let mut n = closest_inner_1;
    loop {
        ctr_clkwise.push(n.id);
        n = n.next(ig);

        if n == closest_inner_2 {
            break;
        }
    }
    n = closest_inner_1;
    loop {
        clkwise.push(n.id);
        n = n.prev(ig);

        if n == closest_inner_2 {
            break;
        }
    }
    if ctr_clkwise.len() < clkwise.len() {
        return ctr_clkwise;
    } else {
        return clkwise;
    }
}

pub(crate) fn avg_change_dumb(tgt: &Node, v: &Vec<&NodeChange>) -> NodeChange {
    let mut nc = NodeChange {
        id: tgt.id,
        cur_x: tgt.x,
        cur_y: tgt.y,
        delta_x: 0.0,
        delta_y: 0.0,
    };
    for i in v {
        nc.delta_x += i.delta_x;
        nc.delta_y += i.delta_y;
    }
    nc.delta_x /= v.len() as f64;
    nc.delta_y /= v.len() as f64;
    nc
}

/*
This fn could have a few versions:
1. n_closest *of the changed nodes* PRE-change
2. n_closest *of changed nodes AND non-changed nodes* PRE-change
3. "    "     POST-change
4. "    "     "      "     POST-change
*/
pub fn n_closest_outers<'a>(n: usize, inner_node: &Node, outer_changes: &'a NodeChangeMap, g: &Graph) -> Vec<&'a NodeChange> {
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
