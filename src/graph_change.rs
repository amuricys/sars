use types::*;
use rand::Rng;
use vector_2d_helpers::{bisecting_vector, norm};
use graph::{distance_between_nodes, available_node_id};
use piston::input::keyboard::Key::Out;
use std::collections::HashMap;

fn apply_change<'a>(g: &'a mut Graph, change: &NodeChange) -> Result<&'a Graph, NodeChange> {
    /* TODO: Not thread safe */
    if g.nodes[change.id].x == change.cur_x && g.nodes[change.id].y == change.cur_y {
        g.nodes[change.id].x = change.cur_x + change.delta_x;
        g.nodes[change.id].y = change.cur_y + change.delta_y;
        Ok(g)
    } else {
        panic!("CARILHO")
    }
}

fn revert_change<'a>(g: &'a mut Graph, change: &NodeChange) -> Result<&'a Graph, NodeChange> {
    /* TODO: Not thread safe */
    if g.nodes[change.id].x == change.cur_x + change.delta_x && g.nodes[change.id].y == change.cur_y + change.delta_y {
        g.nodes[change.id].x = change.cur_x;
        g.nodes[change.id].y = change.cur_y;
        Ok(g)
    } else {
        panic!("CARILHO")
    }
}

pub(crate) fn apply_changes(g: &mut Graph, changes: &NodeChangeMap) {
    /* TODO: This should be atomic if the callers are to be concurrent */
    for (_, change) in changes {
        apply_change(g, &change);
    }
}

pub(crate) fn revert_changes(g: &mut Graph, changes: &NodeChangeMap) {
    /* TODO: This should be atomic if the callers are to be concurrent */
    for (_, change) in changes {
        revert_change(g, &change);
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
        delta_x: x_change,
        delta_y: y_change,
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
            let diff_x = change.delta_x * (how_smooth - dist_traveled_next) / how_smooth;
            let diff_y = change.delta_y * (how_smooth - dist_traveled_next) / how_smooth;
            ret.push(NodeChange { id: cur_next.id, cur_x: cur_next.x, cur_y: cur_next.y, delta_x: diff_x, delta_y: diff_y });
        }
        if !enough_prev {
            let diff_x = change.delta_x * (how_smooth - dist_traveled_prev) / how_smooth;
            let diff_y = change.delta_y * (how_smooth - dist_traveled_prev) / how_smooth;
            ret.push(NodeChange { id: cur_prev.id, cur_x: cur_prev.x, cur_y: cur_prev.y, delta_x: diff_x, delta_y: diff_y });
        }
        if enough_next && enough_prev { break; }
    }
    ret
}

pub fn smooth_change_out2(g: &Graph, change: NodeChange, how_smooth: usize) -> NodeChangeMap {
    let mut ret = HashMap::new();
    ret.insert(change.id, change);
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
            let diff_x = change.delta_x * (how_smooth as f64 - dist_traveled_next as f64) / how_smooth as f64;
            let diff_y = change.delta_y * (how_smooth as f64 - dist_traveled_next as f64) / how_smooth as f64;
            ret.insert(cur_next.id, NodeChange { id: cur_next.id, cur_x: cur_next.x, cur_y: cur_next.y, delta_x: diff_x, delta_y: diff_y });
        }
        if !enough_prev {
            let diff_x = change.delta_x * (how_smooth as f64 - dist_traveled_prev as f64) / how_smooth as f64;
            let diff_y = change.delta_y * (how_smooth as f64 - dist_traveled_prev as f64) / how_smooth as f64;
            ret.insert(cur_prev.id, NodeChange { id: cur_prev.id, cur_x: cur_prev.x, cur_y: cur_prev.y, delta_x: diff_x, delta_y: diff_y });
        }
        if enough_next && enough_prev { break; }
    }
    ret
}

fn assert_cyclicness(ts: &ThickSurface) {
    let fst = &ts.layers[OUTER].nodes[0];
    let mut j = fst.next(&ts.layers[OUTER]);
    println!("Going forward...");
    loop {
        j = j.next(&ts.layers[OUTER]);
        if j == fst { break; }
    }
    println!("k didnt break it. Going backward...");
    loop {
        j = j.prev(&ts.layers[OUTER]);
        if j == fst { break; }
    }
    println!("yay");
    println!("ts: {:?}", ts);
}

pub fn add_node_(ts: &mut ThickSurface, layer_to_which_add: usize, node_addition: NodeAddition) {
    ts.layers[layer_to_which_add].nodes[node_addition.n.next_id].prev_id = node_addition.n.id;
    ts.layers[layer_to_which_add].nodes[node_addition.n.prev_id].next_id = node_addition.n.id;
    ts.layers[layer_to_which_add].nodes.insert(node_addition.n.id, Node { id: node_addition.n.id, ..node_addition.n });

    println!("adding between {} ({:.3}, {:.3}) and {} ({:.3}, {:.3}) (dist: {:.3})\n\n\n",
             node_addition.n.prev_id,
             ts.layers[layer_to_which_add].nodes[node_addition.n.prev_id].x, ts.layers[layer_to_which_add].nodes[node_addition.n.prev_id].y,
             node_addition.n.next_id,
             ts.layers[layer_to_which_add].nodes[node_addition.n.next_id].x, ts.layers[layer_to_which_add].nodes[node_addition.n.next_id].y,
             distance_between_nodes(&ts.layers[layer_to_which_add].nodes[node_addition.n.prev_id], &ts.layers[layer_to_which_add].nodes[node_addition.n.next_id]));

    assert_cyclicness(ts);
}

fn swap_nodes(ts: &mut ThickSurface, layer_from_which_delete: usize, layer_across: usize, deleted_id: usize, last: &Node) {
    /* Deleted node's position will now contain the last node */
    let new_mix = Node {
        id: deleted_id,
        /* The if branches below are for: prev ---> del ---> last ---> last.next */
        next_id: if deleted_id != last.next_id {last.next_id} else {ts.layers[layer_from_which_delete].nodes[deleted_id].next_id},
        prev_id: if deleted_id != last.prev_id  {last.prev_id} else {ts.layers[layer_from_which_delete].nodes[deleted_id].prev_id},
        x: last.x,
        y: last.y
    };
    /* The if branches below are for: prev ----> del/last ----> next. In this case the deleted node's prev, an undeleted one,
       should "skip" the deleted one with the next_id. */
    ts.layers[layer_from_which_delete].nodes[new_mix.next_id].prev_id = if deleted_id == last.id {new_mix.prev_id} else {deleted_id};
    ts.layers[layer_from_which_delete].nodes[new_mix.prev_id].next_id = if deleted_id == last.id {new_mix.next_id} else {deleted_id};
    ts.layers[layer_from_which_delete].nodes[deleted_id] = new_mix;
}

fn simple_delete(ts: &mut ThickSurface, layer_from_which_delete: usize, layer_across: usize, deleted_id: usize) {
    let last = ts.layers[layer_from_which_delete].nodes.last().unwrap().clone();

    /* This fn does mutable magic on the last node, the deleted node, and all involved acrossnesses, and isolates the last node for deletion */
    swap_nodes(ts, layer_from_which_delete, layer_across, deleted_id, &last);

    ts.layers[layer_from_which_delete].nodes.remove(last.id);

    assert_cyclicness(ts);
}

pub fn delete_node_(ts: &mut ThickSurface, layer_from_which_delete: usize, layer_across: usize, (prev_id, next_id): (usize, usize)) {
    /* TODO: We're only doing simple delete for now */
    simple_delete(ts, layer_from_which_delete, layer_across, prev_id)
}


fn direction_vector0(_other_graph: &Graph, change: &NodeChange, _other_graph_changes: &NodeChangeMap) -> (f64, f64) {
    (change.cur_x, change.cur_y)
}

fn direction_vector1(other_graph: &Graph, change: &NodeChange, other_graph_changes: &NodeChangeMap) -> (f64, f64) {
    let changed_nodes_prev = other_graph.nodes[change.id].prev(other_graph);
    let (prev_ref_x, prev_ref_y) = match other_graph_changes.get(&changed_nodes_prev.id) {
        Some(nc) => (nc.cur_x + nc.delta_x, nc.cur_y + nc.delta_y),
        None => (changed_nodes_prev.x, changed_nodes_prev.y)
    };
    let changed_nodes_next = other_graph.nodes[change.id].next(other_graph);
    let (next_ref_x, next_ref_y) = match other_graph_changes.get(&changed_nodes_next.id) {
        Some(nc) => (nc.cur_x + nc.delta_x, nc.cur_y + nc.delta_y),
        None => (changed_nodes_next.x, changed_nodes_next.y)
    };
    /* prev_ref_xy and next_ref_xy are the position along which we want to find the direction vector */
    let (dir_x, dir_y) = bisecting_vector(change.cur_x, change.cur_y, prev_ref_x, prev_ref_y, next_ref_x, next_ref_y);

    (-dir_x * norm(change.delta_x, change.delta_y), -dir_y * norm(change.delta_x, change.delta_y))
}

fn direction_vector2(graph_across: &Graph, other_graph: &Graph, change: &NodeChange, other_graph_changes: &NodeChangeMap, compression_factor: f64) -> (f64, f64) {
    /*TODO: Needs to push each node in the average direction of its correspondent */
    (0.0, 0.0)
}

/* TODO: Should take in map/matrix of correspondences */
fn node_across() -> Node {
    Node {
        id: 0,
        x: 0.0,
        y: 0.0,
        next_id: 0,
        prev_id: 0
    }
}

pub fn changes_from_other_graph(this_graph: &Graph, other_graph: &Graph, other_graph_changes: &NodeChangeMap, compression_factor: f64) -> NodeChangeMap {
    let mut ret = HashMap::new();
    for (_, c) in other_graph_changes {
        let (delta_x, delta_y) = direction_vector2(this_graph, other_graph, c, other_graph_changes, compression_factor);
        let node_across = node_across();
        ret.insert(node_across.id, NodeChange { id: node_across.id, cur_x: node_across.x, cur_y: node_across.y, delta_x, delta_y });
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph::{cyclic_graph_from_coords, circular_graph, area, circular_thick_surface, node_to_add};
    use vec1::Vec1;

    #[test]
    fn change_is_applied_and_reversed() {
        // TODO: This should be generated
        let size_of_test_circ = 40;

        let mut test_circ = circular_graph(0.0, 0.0, 1.0, size_of_test_circ);
        let area_before = area(&test_circ);
        let change = NodeChange { id: 1, cur_x: test_circ.nodes[1].x, cur_y: test_circ.nodes[1].y, delta_x: 70.0, delta_y: 100.0 };

        apply_change(&mut test_circ, &change);
        let area_after_applying = area(&test_circ);

        assert!(area_before < area_after_applying);

        revert_change(&mut test_circ, &change);
        let area_after_reverting = area(&test_circ);

        assert_eq!(area_after_reverting, area_before);
    }

    #[test]
    // fn inner_from_outer_changes() {
    //     let vertical_line = [(0.0, 1.0), (0.0, 0.0), (0.0,-1.0)];
    //     let vertical_line_slightly_to_left = [(-0.2, 1.0), (-0.2, 0.0), (-0.2,-1.0)];
    //     let mut test_outer_graph = cyclic_graph_from_coords(&Vec1::try_from_vec(Vec::from(vertical_line)).unwrap());
    //     let mut test_inner_graph = cyclic_graph_from_coords(&Vec1::try_from_vec(Vec::from(vertical_line_slightly_to_left)).unwrap());
    //     establish_correspondences(&mut test_outer_graph, &mut test_inner_graph);
    //     let mut the_change = HashMap::new();
    //     the_change.insert(1, NodeChange{ id: 1, cur_x: 0.0, cur_y: 0.0, delta_x: 0.5, delta_y: 0.0 });
    //     let the_fuckin_change = changes_from_other_graph(&test_inner_graph, &test_outer_graph, &the_change, 1.0);
    //
    //     println!("{:?}", the_fuckin_change);
    //     assert_eq!(the_fuckin_change[&1 ).unwrap, 0.5);
    //     assert_eq!(the_fuckin_change.get(&1 ).unwrap().delta_y, 0.0);
    //
    // }
    #[test]
    fn random_node_is_changed() {
        // TODO: This should be generated
        let size_of_test_circ = 40;

        let mut test_circ = circular_graph(0.0, 0.0, 1.0, size_of_test_circ);
        let area_before = area(&test_circ);

        let mut rng = rand::thread_rng();
        let change = random_change(&test_circ, (0.01, 0.02), &mut rng);

        apply_change(&mut test_circ, &change);
        let area_after_applying = area(&test_circ);

        assert_ne!(area_before, area_after_applying);
    }

    #[test]
    fn adding_node_leaves_consistent_graph() {
        let size_of_graph = 30;
        let mut circular = circular_thick_surface(1.0, 0.3, size_of_graph);

        // Low addition threshold ensures this adds a node
        let to_add = node_to_add(&circular.layers[OUTER],  &circular.layers[OUTER].nodes[10], &circular.layers[OUTER].nodes[10].next(&circular.layers[OUTER]), 0.000001);
        add_node_(&mut circular, OUTER,  to_add.unwrap());
        let to_add = node_to_add(&circular.layers[OUTER],  &circular.layers[OUTER].nodes[10], &circular.layers[OUTER].nodes[10].next(&circular.layers[OUTER]), 0.000001);
        add_node_(&mut circular, OUTER,  to_add.unwrap());
        let to_add = node_to_add(&circular.layers[OUTER],  &circular.layers[OUTER].nodes[10], &circular.layers[OUTER].nodes[10].next(&circular.layers[OUTER]), 0.000001);
        add_node_(&mut circular, OUTER,  to_add.unwrap());

        let to_add = node_to_add( &circular.layers[INNER], &circular.layers[INNER].nodes[10], &circular.layers[INNER].nodes[10].next(&circular.layers[INNER]), 0.000001);
        add_node_(&mut circular, INNER,  to_add.unwrap());

        /*TODO: FUCK, gotta fix these tests bad. */
    }
}

