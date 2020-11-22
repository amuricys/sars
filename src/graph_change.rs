use types::*;
use rand::Rng;
use vector_2d_helpers::{direction_vector};
use graph::{distance_between_nodes};
use graphics::modular_index::next;
use piston::input::keyboard::Key::Out;

fn apply_change(g: &mut Graph, change: NodeChange) -> Result<&Graph, NodeChange> {
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

pub(crate) fn apply_changes(g: &mut Graph, changes: &Vec<NodeChange>) {
    /* TODO: This should be atomic if the callers are to be concurrent */
    for change in changes {
        apply_change(g, change.clone());
    }
}

pub(crate) fn revert_changes(g: &mut Graph, changes: &Vec<NodeChange>) {
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
            ret.push(NodeChange { id: cur_next.id, cur_x: cur_next.x, cur_y: cur_next.y, new_x: cur_next.x + diff_x, new_y: cur_next.y + diff_y });
        }
        if !enough_prev {
            let diff_x = (change.new_x - change.cur_x) * (how_smooth - dist_traveled_prev) / how_smooth;
            let diff_y = (change.new_y - change.cur_y) * (how_smooth - dist_traveled_prev) / how_smooth;
            ret.push(NodeChange { id: cur_prev.id, cur_x: cur_prev.x, cur_y: cur_prev.y, new_x: cur_prev.x + diff_x, new_y: cur_prev.y + diff_y });
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
            let diff_x = (change.new_x - change.cur_x) * (how_smooth as f64 - dist_traveled_next as f64) / how_smooth as f64;
            let diff_y = (change.new_y - change.cur_y) * (how_smooth as f64 - dist_traveled_next as f64) / how_smooth as f64;
            ret.push(NodeChange { id: cur_next.id, cur_x: cur_next.x, cur_y: cur_next.y, new_x: cur_next.x + diff_x, new_y: cur_next.y + diff_y });
        }
        if !enough_prev {
            let diff_x = (change.new_x - change.cur_x) * (how_smooth as f64 - dist_traveled_prev as f64) / how_smooth as f64;
            let diff_y = (change.new_y - change.cur_y) * (how_smooth as f64 - dist_traveled_prev as f64) / how_smooth as f64;
            ret.push(NodeChange { id: cur_prev.id, cur_x: cur_prev.x, cur_y: cur_prev.y, new_x: cur_prev.x + diff_x, new_y: cur_prev.y + diff_y });
        }
        if enough_next && enough_prev { break; }
    }
    ret
}

fn assert_acrossness(ts: &ThickSurface) {
    let fst = &ts.layers[OUTER].nodes[0];
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

fn available_edge_id(g: &Graph) -> usize {
    /* Graph edges should be Option(Edge)s */
    g.edges.len()
}


pub fn add_node_(ts: &mut ThickSurface, layer_to_which_add: usize, layer_across: usize, node_addition: NodeAddition) {
    let actual_node_id = available_node_id(&ts.layers[layer_to_which_add]);
    let actual_edge_id = available_edge_id(&ts.layers[layer_to_which_add]);

    for across in &node_addition.n.acrossness {
        ts.layers[layer_across].nodes[*across].acrossness.push(actual_node_id);
    }

    println!("Adding node {:?}", node_addition);
    let out_index = ts.layers[layer_to_which_add].nodes[node_addition.prev_id].out;
    ts.layers[layer_to_which_add].edges[out_index].target = actual_node_id;
    ts.layers[layer_to_which_add].nodes[node_addition.next_id].inc = actual_edge_id;
    ts.layers[layer_to_which_add].nodes.push( Node {id: actual_node_id, ..node_addition.n});
    ts.layers[layer_to_which_add].edges.push(EdgeSameSurface{id: actual_edge_id, ..node_addition.e});

    println!("adding between {} ({:.3}, {:.3}) and {} ({:.3}, {:.3}) (dist: {:.3})",
             node_addition.prev_id,
             ts.layers[layer_to_which_add].nodes[node_addition.prev_id].x, ts.layers[layer_to_which_add].nodes[node_addition.prev_id].y,
             node_addition.next_id,
             ts.layers[layer_to_which_add].nodes[node_addition.next_id].x, ts.layers[layer_to_which_add].nodes[node_addition.next_id].y,
             distance_between_nodes(&ts.layers[layer_to_which_add].nodes[node_addition.prev_id], &ts.layers[layer_to_which_add].nodes[node_addition.next_id]));
    assert_acrossness(ts);
}

/* TODO (but this is actually far from a next step):
   This way of doing this is actually probably bad, because it doesn't take simultaneous changes into account.
   Meaning an inner node is calculating its position in comparison only to its immediate outer correspondent, without
   considering the correspondent's neighbors, which probably also changed due to smoothing. */
pub fn changes_from_other_graph(this_graph: &Graph, other_graph: &Graph, other_graph_changes: &Vec<NodeChange>, compression_factor: f64) -> Vec<NodeChange> {
    let mut ret = Vec::new();
    for c in other_graph_changes {
        /* TODO: Compression, look at across, better understand. LOL it's breaking because we add nodes to outside. LOOK AT ACROSS */
        let cur_node = &other_graph.nodes[c.id];
        let mut nodes_across = Vec::new();

        /* When a single node across exists, the below should work */
        let acr_id = other_graph.nodes[c.id].acrossness[0];
        let node_across = &this_graph.nodes[acr_id];
        let (prev_node, next_node) = (cur_node.prev(other_graph), cur_node.next(other_graph));

        let dist = distance_between_nodes(cur_node, node_across);
        let (dir_x, dir_y) = direction_vector(cur_node.x, cur_node.y, prev_node.x, prev_node.y, next_node.x, next_node.y);

        let (delta_x, delta_y) = (-dir_x * dist * compression_factor, -dir_y * dist * compression_factor);
        nodes_across.push(NodeChange { id: node_across.id, cur_x: node_across.x, cur_y: node_across.y, new_x: c.new_x + delta_x, new_y: c.new_y + delta_y });

        nodes_across = nodes_across.iter().map(|x| NodeChange { new_x: x.cur_x + ((x.new_x - x.cur_x) / nodes_across.len() as f64), ..*x }).collect();
        ret.append(&mut nodes_across);
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph::{circular_graph, area, circular_thick_surface, node_to_add};

    #[test]
    fn change_is_applied_and_reversed() {
        // TODO: This should be generated
        let size_of_test_circ = 40;

        let mut test_circ = circular_graph(0.0, 0.0, 1.0, size_of_test_circ);
        let area_before = area(&test_circ);
        let change = NodeChange { id: 1, cur_x: test_circ.nodes[1].x, cur_y: test_circ.nodes[1].y, new_x: 70.0, new_y: 100.0 };

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

        let mut rng = rand::thread_rng();
        let change = random_change(&test_circ, (0.01, 0.02), &mut rng);

        apply_change(&mut test_circ, change);
        let area_after_applying = area(&test_circ);

        assert_ne!(area_before, area_after_applying);
    }

    #[test]
    fn adding_node_leaves_consistent_graph() {
        let size_of_graph = 30;
        let mut circular = circular_thick_surface(1.0, 0.3, size_of_graph);

        // Low addition threshold ensures this adds a node
        let node_to_add = node_to_add(&circular.layers[OUTER], &circular.layers[OUTER].nodes[10], &circular.layers[OUTER].nodes[10].next(&circular.layers[OUTER]), 0.000001);
        println!("{:?}", node_to_add);
        add_node_(&mut circular, OUTER, node_to_add.unwrap());
        assert_eq!(node_to_add, node_to_add)
    }
}

