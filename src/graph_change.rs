use types::*;
use rand::Rng;
use vector_2d_helpers::{direction_vector, distance_between_nodes};
use graph_change::NeighborlyStatus::{FirstToSecond, SecondToFirst, NotNeighbors};
use std::collections::HashSet;

pub fn apply_change(g: &mut Graph, change: NodeChange) -> Result<&Graph, NodeChange> {
    /* TODO: Not thread safe */
    if g.nodes[change.id].x == change.cur_x && g.nodes[change.id].y == change.cur_y {
        g.nodes[change.id].x = change.new_x;
        g.nodes[change.id].y = change.new_y;
        Ok(g)
    } else {
        Err(change)
    }
}

fn revert_change(g: &mut Graph, change: NodeChange) -> Result<&Graph, NodeChange> {
    /* TODO: Not thread safe */
    if g.nodes[change.id].x == change.new_x && g.nodes[change.id].y == change.new_y {
        g.nodes[change.id].x = change.cur_x;
        g.nodes[change.id].y = change.cur_y;
        Ok(g)
    } else {
        Err(change)
    }
}

pub (crate) fn apply_changes(g: &mut Graph, changes: &Vec<NodeChange>) {
    /* TODO: This should be atomic if the callers are to be concurrent */
    for change in changes {
        apply_change(g, change.clone());
    }
}

pub (crate) fn revert_changes(g: &mut Graph, changes: &Vec<NodeChange>) {
    /* TODO: This should be atomic if the callers are to be concurrent */
    for change in changes {
        revert_change(g, change.clone());
    }
}

fn random_node(g: &Graph, rng: &mut rand::rngs::ThreadRng) -> NodeIndex {
    let annoyingly_needed_due_to_rusts_type_inference: usize = rng.gen();
    annoyingly_needed_due_to_rusts_type_inference % g.nodes.len()
}

pub fn random_change(g: &Graph, (low, high): (f64, f64), rng: &mut rand::rngs::ThreadRng) -> NodeChange {
    let to_change = random_node(g, rng);
    let x_change = rng.gen_range(low, high);
    let y_change = rng.gen_range(low, high);
    NodeChange {
        id: to_change,
        cur_x: g.nodes[to_change].x,
        cur_y: g.nodes[to_change].y,
        new_x: g.nodes[to_change].x + x_change,
        new_y: g.nodes[to_change].y + y_change,
    }
}

/* TODO: These fns are almost the same. There is a smarter way to do this */
pub fn smooth_change_out(g: &Graph, change: NodeChange, how_smooth: f64) -> Vec<NodeChange> {
    let mut ret = Vec::new();
    ret.push(change);
    let mut dist_traveled_prev = 0.0;
    let mut dist_traveled_next = 0.0;
    let mut cur_next = &g.nodes[change.id];
    let mut cur_prev = &g.nodes[change.id];

    loop {
        cur_next = cur_next.next(g);
        cur_prev = cur_prev.prev(g);

        dist_traveled_next = dist_traveled_next + distance_between_nodes(&g.nodes[change.id], cur_next);
        dist_traveled_prev = dist_traveled_prev + distance_between_nodes(&g.nodes[change.id], cur_prev);

        let enough_next = dist_traveled_next > how_smooth;
        let enough_prev = dist_traveled_prev > how_smooth;

        if !enough_next {
            let diff_x = (change.new_x - change.cur_x) * (how_smooth - dist_traveled_next) / how_smooth;
            let diff_y = (change.new_y - change.cur_y) * (how_smooth - dist_traveled_next) / how_smooth;
            ret.push(NodeChange{id: cur_next.id, cur_x: cur_next.x, cur_y: cur_next.y, new_x: cur_next.x + diff_x, new_y: cur_next.y + diff_y});
        }
        if !enough_prev {
            let diff_x = (change.new_x - change.cur_x) * (how_smooth - dist_traveled_prev) / how_smooth;
            let diff_y = (change.new_y - change.cur_y) * (how_smooth - dist_traveled_prev) / how_smooth;
            ret.push(NodeChange{id: cur_prev.id, cur_x: cur_prev.x, cur_y: cur_prev.y, new_x: cur_prev.x + diff_x, new_y: cur_prev.y + diff_y});
        }
        if enough_next && enough_prev { break; }
    }
    ret
}

pub fn smooth_change_out2(g: &Graph, change: NodeChange, how_smooth: usize) -> Vec<NodeChange> {
    let mut ret = Vec::new();
    ret.push(change);
    let mut dist_traveled_prev = 0;
    let mut dist_traveled_next = 0;
    let mut cur_next = &g.nodes[change.id];
    let mut cur_prev = &g.nodes[change.id];

    loop {
        cur_next = cur_next.next(g);
        cur_prev = cur_prev.prev(g);

        dist_traveled_next = dist_traveled_next + 1;
        dist_traveled_prev = dist_traveled_prev + 1;

        let enough_next = dist_traveled_next > how_smooth;
        let enough_prev = dist_traveled_prev > how_smooth;

        if !enough_next {
            let diff_x = (change.new_x - change.cur_x) * (how_smooth as f64 - dist_traveled_next as f64) / how_smooth  as f64;
            let diff_y = (change.new_y - change.cur_y) * (how_smooth as f64 - dist_traveled_next as f64) / how_smooth as f64;
            ret.push(NodeChange{id: cur_next.id, cur_x: cur_next.x, cur_y: cur_next.y, new_x: cur_next.x + diff_x, new_y: cur_next.y + diff_y});
        }
        if !enough_prev {
            let diff_x = (change.new_x - change.cur_x) * (how_smooth as f64 - dist_traveled_prev as f64) / how_smooth as f64;
            let diff_y = (change.new_y - change.cur_y) * (how_smooth as f64 - dist_traveled_prev as f64) / how_smooth as f64;
            ret.push(NodeChange{id: cur_prev.id, cur_x: cur_prev.x, cur_y: cur_prev.y, new_x: cur_prev.x + diff_x, new_y: cur_prev.y + diff_y});
        }
        if enough_next && enough_prev { break; }
    }
    ret
}

#[derive(Debug, PartialEq)]
enum NeighborlyStatus {
    FirstToSecond,
    SecondToFirst,
    NotNeighbors
}

fn neighborly_status(g: &Graph, id1: usize, id2: usize) -> NeighborlyStatus {
    if g.nodes[id1].next(g).id == id2 && g.nodes[id2].prev(g).id == id1 {
        FirstToSecond
    } else if g.nodes[id1].prev(g).id == id2 && g.nodes[id2].next(g).id == id1 {
        SecondToFirst
    } else {
        NotNeighbors
    }
}

fn lookup_node_id(g: &Graph) -> usize {
    /* Graph nodes should be Option(Node)s */
    g.nodes.len()
}

fn lookup_edge_id(g: &Graph) -> usize {
    /* Graph edges should be Option(Edge)s */
    g.edges.len()
}

fn add_node_(g: &mut Graph, left_id: usize, right_id: usize){
    let new_node_id = lookup_node_id(&g);
    let new_edge_id = lookup_edge_id(&g);
    let new_node = Node {
        id: new_node_id,
        x: (g.nodes[left_id].x + g.nodes[right_id].x) / 2.0,
        y: (g.nodes[left_id].y + g.nodes[right_id].y) / 2.0,
        inc: g.edges[g.nodes[left_id].out].id,
        out: new_edge_id,
        across: HashSet::new() // TODO this will break
    };
    let new_edge = EdgeSameSurface {
        id: new_edge_id,
        source: new_node_id,
        target: g.nodes[right_id].id
    };
    g.edges[g.nodes[left_id].out].target = new_node_id;
    g.nodes[right_id].inc = new_edge_id;
    g.nodes.push(new_node);
    g.edges.push(new_edge);
}

pub fn add_node(g: &mut Graph, id1: usize, id2: usize) -> Result<(), &str>{
    match neighborly_status(g, id1, id2) {
        FirstToSecond => Ok(add_node_(g, id1, id2)),
        SecondToFirst => Ok(add_node_(g, id2, id1)),
        NotNeighbors => Err("Tried adding node between two non-neighbors")
    }
}

/* NEXTSTEP:
   This way of doing this is actually probably bad, because it doesn't take simultaneous changes into account.
   Meaning an inner node is calculating its position in comparison only to its immediate outer correspondent, without
   considering the correspondent's neighbors, which probably also changed due to smoothing. */
pub fn changes_from_other_graph(this_graph: &Graph, other_graph: &Graph, other_graph_changes: &Vec<NodeChange>, compression_factor: f64) -> Vec<NodeChange> {
    let mut ret = Vec::new();
    for c in other_graph_changes {
        /* TODO: Compression, look at across, better understand. LOL it's breaking because we add nodes to outside. LOOK AT ACROSS */
        let cur_node = &other_graph.nodes[c.id];
        let node_across = &this_graph.nodes[c.id];
        let (prev_node, next_node) = (cur_node.prev(other_graph), cur_node.next(other_graph));

        let dist= distance_between_nodes(cur_node, node_across);
        let (dir_x, dir_y) = direction_vector(cur_node.x, cur_node.y, prev_node.x, prev_node.y, next_node.x, next_node.y);

        let (delta_x, delta_y) = (-dir_x * dist * compression_factor, -dir_y * dist * compression_factor);
        ret.push(NodeChange{id: node_across.id, cur_x: node_across.x, cur_y: node_across.y, new_x: c.new_x + delta_x, new_y: c.new_y + delta_y})
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph::{circular_graph, area};

    #[test]
    fn change_is_applied_and_reversed() {
        // TODO: This should be generated
        let size_of_test_circ = 40;

        let mut test_circ = circular_graph(0.0, 0.0, 1.0, size_of_test_circ);
        let area_before = area(&test_circ);
        let change = NodeChange {id: 1, cur_x: test_circ.nodes[1].x, cur_y: test_circ.nodes[1].y, new_x: 70.0, new_y: 100.0};

        apply_change(&mut test_circ, change);
        let area_after_applying = area(&test_circ);

        assert!(area_before < area_after_applying);

        revert_change(&mut test_circ, change);
        let area_after_reverting = area(&test_circ);

        assert_eq!(area_after_reverting, area_before);
    }

    #[test]
    fn random_node_is_changed() {
        // TODO: This should be generated
        let size_of_test_circ = 40;

        let mut test_circ = circular_graph(0.0, 0.0, 1.0, size_of_test_circ);
        let area_before = area(&test_circ);

        let mut rng =  rand::thread_rng();
        let change = random_change(&test_circ, (0.01, 0.02), &mut rng);

        apply_change(&mut test_circ, change);
        let area_after_applying = area(&test_circ);

        assert_ne!(area_before, area_after_applying);
    }

}

