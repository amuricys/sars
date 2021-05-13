use graph::types::{NodeChangeMap, Graph, Node, NodeChange};
use graph::{closest_node_to_some_point, distance_between_points};

pub(crate) fn most_prev_next<'a>(ncm: &NodeChangeMap, g: &'a Graph) -> (&'a Node, &'a Node){
    let (_, most_next) = ncm.unwrap().iter().find(|(_, v)| {
        match ncm.get(&g.next(v.id).id) {
            None => true,
            _ => false
        }
    }).unwrap();
    let (_, most_prev) = ncm.unwrap().iter().find(|(_, v)| {
        match ncm.get(&g.prev(v.id).id) {
            None => true,
            _ => false
        }
    }).unwrap();
    (&g.nodes[most_prev.id], &g.nodes[most_next.id])
}

pub(crate) fn closest_internal_nodes<'a>(most_outer_prev: &Node, most_outer_next: &Node, ig: &'a Graph) -> (&'a Node, &'a Node) {
    (
        closest_node_to_some_point(ig, most_outer_prev.x, most_outer_prev.y),
        closest_node_to_some_point(ig, most_outer_next.x, most_outer_next.y)
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
        delta_y: 0.0
    };
    for i in v {
        nc.delta_x += i.delta_x;
        nc.delta_y += i.delta_y;
    }
    nc.delta_x /= v.len() as f64;
    nc.delta_y /= v.len() as f64;
    nc
}

#[derive(Copy, Clone)]
enum PrePost { Pre, Post }
fn avg_change_essence(tgt: &Node, v: &Vec<&NodeChange>, pp: PrePost) -> NodeChange {
    /* Sum Of all Distances from the changed nodes to the target node */
    let sod = v.iter().fold(0.0, |acc, x|
        acc + distance_between_points(
            x.cur_x + match pp { PrePost::Pre => 0.0, PrePost::Post => x.delta_x },
            x.cur_y  + match pp { PrePost::Pre => 0.0, PrePost::Post => x.delta_y },
            tgt.x, tgt.y
        )
    );

    let mut nc = NodeChange {
        id: tgt.id,
        cur_x: tgt.x,
        cur_y: tgt.y,
        delta_x: 0.0,
        delta_y: 0.0
    };
    let mut running_total_of_something = 0.0;
    for i in v {
        running_total_of_something += (sod - distance_between_points(i.cur_x + i.delta_x, i.cur_y + i.delta_y, tgt.x, tgt.y));
        nc.delta_x += i.delta_x * (sod - distance_between_points(i.cur_x + i.delta_x, i.cur_y + i.delta_y, tgt.x, tgt.y));
        nc.delta_y += i.delta_y * (sod - distance_between_points(i.cur_x + i.delta_x, i.cur_y + i.delta_y, tgt.x, tgt.y));;
    }
    nc.delta_x /= running_total_of_something;
    nc.delta_y /= running_total_of_something;
    nc
}

/* Averages based on the distance from the changed nodes' PRE-change positions to the target */
pub fn avg_change_smart(tgt: &Node, v: &Vec<&NodeChange>) -> NodeChange {
    avg_change_essence(tgt, v, PrePost::Pre)
}

/* Averages based on the distance from the changed nodes' POST change positions to the target */
pub fn avg_change_smarter(tgt: &Node, v: &Vec<&NodeChange>) -> NodeChange {
    avg_change_essence(tgt, v, PrePost::Post)

}