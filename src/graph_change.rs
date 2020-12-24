use types::*;
use rand::Rng;
use vector_2d_helpers::{bisecting_vector, norm};
use graph::{distance_between_nodes};
use graphics::modular_index::next;
use piston::input::keyboard::Key::Out;
use std::collections::HashMap;

fn apply_change<'a>(g: &'a mut Graph, change: &NodeChange) -> Result<&'a Graph, NodeChange> {
    /* TODO: Not thread safe */
    if g.nodes.get(&change.id).unwrap().x == change.cur_x && g.nodes.get(&change.id).unwrap().y == change.cur_y {
        g.nodes.get_mut(&change.id).unwrap().x = change.cur_x + change.delta_x;
        g.nodes.get_mut(&change.id).unwrap().y = change.cur_y + change.delta_y;
        Ok(g)
    } else {
        panic!("CARILHO")
    }
}

fn revert_change<'a>(g: &'a mut Graph, change: &NodeChange) -> Result<&'a Graph, NodeChange> {
    /* TODO: Not thread safe */
    if g.nodes.get(&change.id).unwrap().x == change.cur_x + change.delta_x && g.nodes.get(&change.id).unwrap().y == change.cur_y + change.delta_y {
        g.nodes.get_mut(&change.id).unwrap().x = change.cur_x;
        g.nodes.get_mut(&change.id).unwrap().y = change.cur_y;
        Ok(g)
    } else {
        panic!("CARILHO")
    }
}

pub(crate) fn apply_changes(g: &mut Graph, changes: &HashMap<usize, NodeChange>) {
    /* TODO: This should be atomic if the callers are to be concurrent */
    for (_, change) in changes {
        apply_change(g, &change);
    }
}

pub(crate) fn revert_changes(g: &mut Graph, changes: &HashMap<usize, NodeChange>) {
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
        cur_x: g.nodes.get(&to_change).unwrap().x,
        cur_y: g.nodes.get(&to_change).unwrap().y,
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
    let mut cur_next = g.nodes.get(&change.id).unwrap();
    let mut cur_prev = g.nodes.get(&change.id).unwrap();

    loop {
        cur_next = cur_next.next(g);
        cur_prev = cur_prev.prev(g);

        dist_traveled_next = dist_traveled_next + distance_between_nodes(g.nodes.get(&change.id).unwrap(), cur_next);
        dist_traveled_prev = dist_traveled_prev + distance_between_nodes(g.nodes.get(&change.id).unwrap(), cur_prev);

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

pub fn smooth_change_out2(g: &Graph, change: NodeChange, how_smooth: usize) -> HashMap<usize, NodeChange> {
    let mut ret = HashMap::new();
    ret.insert(change.id, change);
    let mut dist_traveled_prev = 0;
    let mut dist_traveled_next = 0;
    let mut cur_next = g.nodes.get(&change.id).unwrap();
    let mut cur_prev = g.nodes.get(&change.id).unwrap();

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
            ret.insert(cur_prev.id,NodeChange { id: cur_prev.id, cur_x: cur_prev.x, cur_y: cur_prev.y, delta_x: diff_x, delta_y: diff_y });
        }
        if enough_next && enough_prev { break; }
    }
    ret
}

fn assert_acrossness(ts: &ThickSurface) {
    let (_, fst) = ts.layers[OUTER].nodes.iter().nth(0).unwrap();
    let mut j = fst.next(&ts.layers[OUTER]);
    println!("Going forward...");
    loop {
        j = j.next(&ts.layers[OUTER]);
        if j == fst { break }
    }
    println!("k didnt fuck it up. Going backward...");
    loop {
        j = j.prev(&ts.layers[OUTER]);
        if j == fst { break }
    }
    println!("yay")
}

fn available_node_id(g: &Graph) -> usize {
    /* Graph nodes should be Option(Node)s */
    g.nodes.len()
}


pub fn add_node_(ts: &mut ThickSurface, layer_to_which_add: usize, layer_across: usize, node_addition: NodeAddition) {
    let actual_node_id = available_node_id(&ts.layers[layer_to_which_add]);

    for across in &node_addition.n.acrossness {
        ts.layers[layer_across].nodes.get_mut(across).unwrap().acrossness.push(actual_node_id);
    }

    println!("Adding node {:?}", node_addition);
    ts.layers[layer_to_which_add].nodes.get_mut(&node_addition.n.next_id).unwrap().prev_id = node_addition.n.id;
    ts.layers[layer_to_which_add].nodes.get_mut(&node_addition.n.prev_id).unwrap().next_id = node_addition.n.id;
    ts.layers[layer_to_which_add].nodes.insert(actual_node_id,  Node {id: actual_node_id, ..node_addition.n});

    println!("adding between {} ({:.3}, {:.3}) and {} ({:.3}, {:.3}) (dist: {:.3})",
             node_addition.n.prev_id,
             ts.layers[layer_to_which_add].nodes.get(&node_addition.n.prev_id).unwrap().x, ts.layers[layer_to_which_add].nodes.get(&node_addition.n.prev_id).unwrap().y,
             node_addition.n.next_id,
             ts.layers[layer_to_which_add].nodes.get(&node_addition.n.next_id).unwrap().x, ts.layers[layer_to_which_add].nodes.get(&node_addition.n.next_id).unwrap().y,
             distance_between_nodes(&ts.layers[layer_to_which_add].nodes.get(&node_addition.n.prev_id).unwrap(), &ts.layers[layer_to_which_add].nodes.get(&node_addition.n.next_id).unwrap()));
    assert_acrossness(ts);
}

fn simple_delete (ts: &mut ThickSurface, layer_from_which_delete: usize, layer_across: usize, deleted_id: usize){
    // let deleted = ts.layers[layer_from_which_delete].nodes.get(&deleted_id).unwrap();
    // let deleteds_next = deleted.next(&ts.layers[layer_from_which_delete]);
    // let deleteds_prev = deleted.prev(&ts.layers[layer_from_which_delete]);
    // let deleteds_out_edge = deleted.prev_id;
    // let deleteds_inc_edge = deleted.next_id;
    // let deleteds_next_inc_edge = deleteds_next.next_id;
    // let deleteds_prev_out_edge = deleteds_prev.prev_id;
    //
    // for acr in &deleted.acrossness {
    //     let ind = {
    //         let mut c = 0;
    //         for i in &ts.layers[layer_across].nodes.get(acr).unwrap().acrossness {
    //             if i == acr { break } else { c += 1; }
    //         }
    //         c
    //     };
    // //    ts.layers[layer_across].nodes.get_mut(acr).unwrap().acrossness.try_remove(ind);
    // }
    //
    // let (deleteds_next_id, deleteds_prev_id) = (deleteds_next.id, deleteds_prev.id);
    // ts.layers[layer_from_which_delete].nodes.get_mut(&deleteds_next_id).unwrap().next_id = deleteds_next_inc_edge;
    // ts.layers[layer_from_which_delete].nodes.get_mut(&deleteds_prev_id).unwrap().prev_id = deleteds_next_inc_edge;

    // *Now just delete node and edges*
}

pub fn delete_node_(ts: &mut ThickSurface, layer_from_which_delete: usize, layer_across: usize, (prev_id, next_id): (usize, usize)) {
    fn not_the_only_across (j: &Node, g_across: &Graph) -> bool {
        j.acrossness.iter().all(|x| {g_across.nodes.get(x).unwrap().acrossness.len() > 1})
    }

    let (prev, next) = (ts.layers[layer_from_which_delete].nodes.get(&prev_id).unwrap(), ts.layers[layer_from_which_delete].nodes.get(&next_id).unwrap());
    if not_the_only_across(prev, &ts.layers[layer_across]) {
        simple_delete(ts, layer_from_which_delete, layer_across,prev_id)
    } else if not_the_only_across(next, &ts.layers[layer_across]) {
        simple_delete(ts, layer_from_which_delete, layer_across, next_id)
    }

}


fn direction_vector0(_other_graph: &Graph, change: &NodeChange, _other_graph_changes: &HashMap<usize, NodeChange>) -> (f64, f64) {
    (change.cur_x, change.cur_y)
}

fn direction_vector1(other_graph: &Graph, change: &NodeChange, other_graph_changes: &HashMap<usize, NodeChange>) -> (f64, f64) {
    let changed_nodes_prev = other_graph.nodes.get(&change.id).unwrap().prev(other_graph);
    let (prev_ref_x, prev_ref_y) = match other_graph_changes.get(&changed_nodes_prev.id) {
        Some(nc) => (nc.cur_x + nc.delta_x, nc.cur_y + nc.delta_y),
        None => (changed_nodes_prev.x, changed_nodes_prev.y)
    };
    let changed_nodes_next = other_graph.nodes.get(&change.id).unwrap().next(other_graph);
    let (next_ref_x, next_ref_y) = match other_graph_changes.get(&changed_nodes_next.id) {
        Some(nc) => (nc.cur_x + nc.delta_x, nc.cur_y + nc.delta_y),
        None => (changed_nodes_next.x, changed_nodes_next.y)
    };
    /* prev_ref_xy and next_ref_xy are the position along which we want to find the direction vector */
    let (dir_x, dir_y) = bisecting_vector(change.cur_x, change.cur_y, prev_ref_x, prev_ref_y, next_ref_x, next_ref_y);

    (- dir_x * norm(change.delta_x, change.delta_y), - dir_y * norm(change.delta_x, change.delta_y))
}

fn direction_vector2(graph_across: &Graph, other_graph: &Graph, change: &NodeChange, other_graph_changes: &HashMap<usize, NodeChange>, compression_factor: f64) -> (f64, f64) {
    let changed_nodes_prev = other_graph.nodes.get(&change.id).unwrap().prev(other_graph);
    let (prev_ref_x, prev_ref_y) = match other_graph_changes.get(&changed_nodes_prev.id) {
        //Some(nc) => (nc.cur_x + nc.delta_x, nc.cur_y + nc.delta_y),
        _ => (changed_nodes_prev.x, changed_nodes_prev.y)
    };
    let changed_nodes_next = other_graph.nodes.get(&change.id).unwrap().next(other_graph);
    let (next_ref_x, next_ref_y) = match other_graph_changes.get(&changed_nodes_next.id) {
        //Some(nc) => (nc.cur_x + nc.delta_x, nc.cur_y + nc.delta_y),
        _ => (changed_nodes_next.x, changed_nodes_next.y)
    };
    /* prev_ref_xy and next_ref_xy are the position along which we want to find the direction vector */
    let (dir_x, dir_y) = bisecting_vector(change.cur_x, change.cur_y, next_ref_x, next_ref_y, prev_ref_x, prev_ref_y);

    let node_across = graph_across.nodes.get(&other_graph.nodes.get(&change.id).unwrap().acrossness[0]).unwrap();
    let dist = distance_between_nodes(&node_across, other_graph.nodes.get(&change.id).unwrap());

    let (desired_pos_x, desired_pos_y) = ((change.cur_x + change.delta_x) + dir_x * dist * compression_factor, (change.cur_y + change.delta_y) + dir_y * dist  * compression_factor);

    (- node_across.x + desired_pos_x, - node_across.y + desired_pos_y)
}

/* TODO: other_graph_changes should become a HashMap<usize, NodeChange>. This allows it to find the soon-to-be changed
    versions of the outer changed nodes, calculate what the delta of the nodes across is in relation to _that_ position,
    and then push it in that direction with weight (1 / acrossness_len) */
pub fn changes_from_other_graph(this_graph: &Graph, other_graph: &Graph, other_graph_changes: &HashMap<usize, NodeChange>, compression_factor: f64) -> HashMap<usize, NodeChange> {
    let mut ret = HashMap::new();
    for (_, c) in other_graph_changes {
        let (delta_x, delta_y) = direction_vector2(this_graph, other_graph, c, other_graph_changes, compression_factor);

        /* This should be done for each node across the changed one */
        for acr_id in &other_graph.nodes.get(&c.id).unwrap().acrossness {
            let node_across = this_graph.nodes.get(acr_id).unwrap();
            /*The line below normalizes this change: if a change is made to one of 3 of an inner node's acrosses, that change should only push it by 1/3 */
            let (delta_x, delta_y) = (delta_x / node_across.acrossness.len() as f64, delta_y / node_across.acrossness.len() as f64);
            ret.insert(node_across.id, NodeChange { id: node_across.id, cur_x: node_across.x, cur_y: node_across.y, delta_x, delta_y});
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph::{cyclic_graph_from_coords, circular_graph, area, circular_thick_surface, node_to_add, establish_correspondences};
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
    //     assert_eq!(the_fuckin_change.get(&1 ).unwrap().delta_x, 0.5);
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
        let to_add = node_to_add(&circular.layers[OUTER], &circular.layers[INNER], &circular.layers[OUTER].nodes[10], &circular.layers[OUTER].nodes[10].next(&circular.layers[OUTER]), 0.000001);
        add_node_(&mut circular, OUTER, INNER, to_add.unwrap());
        let to_add = node_to_add(&circular.layers[OUTER], &circular.layers[INNER], &circular.layers[OUTER].nodes[10], &circular.layers[OUTER].nodes[10].next(&circular.layers[OUTER]), 0.000001);
        add_node_(&mut circular, OUTER, INNER, to_add.unwrap());
        let to_add = node_to_add(&circular.layers[OUTER], &circular.layers[INNER], &circular.layers[OUTER].nodes[10], &circular.layers[OUTER].nodes[10].next(&circular.layers[OUTER]), 0.000001);
        add_node_(&mut circular, OUTER, INNER, to_add.unwrap());

        let to_add = node_to_add(&circular.layers[INNER], &circular.layers[OUTER], &circular.layers[INNER].nodes[10], &circular.layers[INNER].nodes[10].next(&circular.layers[INNER]), 0.000001);
        add_node_(&mut circular, INNER, OUTER, to_add.unwrap());

        for n in &circular.layers[OUTER].nodes {
            for acr in &n.acrossness {
                let mut found = false;
                for acr_acr in &circular.layers[INNER].nodes[*acr].acrossness {
                    if *acr_acr == n.id { found = true; break }
                }
                if !found { panic!("wtf!") }
            }
        }

        for n in &circular.layers[INNER].nodes {
            for acr in &n.acrossness {
                let mut found = false;
                for acr_acr in &circular.layers[OUTER].nodes[*acr].acrossness {
                    if *acr_acr == n.id { found = true; break }
                }
                if !found { panic!("wtf!") }
            }
        }
        assert_eq!(1.0, 1.0)
    }
}

