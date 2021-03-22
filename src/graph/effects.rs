use graph::{distance_between_nodes, distance_between_points};

use rand::Rng;

use graph::types::*;
use linalg_helpers::{bisecting_vector, lines_intersection, norm};
use stitcher::types::Stitching;
use vec1::Vec1;

fn apply_change(g: &mut Graph, change: &NodeChange) {
    /* TODO: Not thread safe */
    if g.nodes[change.id].x == change.cur_x && g.nodes[change.id].y == change.cur_y {
        g.nodes[change.id].x = change.cur_x + change.delta_x;
        g.nodes[change.id].y = change.cur_y + change.delta_y;
    } else {
        panic!("CARILHO")
    }
}

fn revert_change(g: &mut Graph, change: &NodeChange) {
    /* TODO: Not thread safe */
    if g.nodes[change.id].x == change.cur_x + change.delta_x && g.nodes[change.id].y == change.cur_y + change.delta_y {
        g.nodes[change.id].x = change.cur_x;
        g.nodes[change.id].y = change.cur_y;
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

fn mk_change(node: &Node, other_change: NodeChange, how_smooth_f64: f64, dist_traveled: f64) -> NodeChange {
    let diff_x = other_change.delta_x * (how_smooth_f64 - dist_traveled) / how_smooth_f64;
    let diff_y = other_change.delta_y * (how_smooth_f64 - dist_traveled) / how_smooth_f64;
    NodeChange {
        id: node.id,
        cur_x: node.x,
        cur_y: node.y,
        delta_x: diff_x,
        delta_y: diff_y,
    }
}

pub fn smooth_change_out(g: &Graph, change: NodeChange, how_smooth: Smooth<usize, f64>) -> NodeChangeMap {
    let mut ret = NodeChangeMap::new();
    ret.insert(change.id, change);

    let mut dist_traveled_prev = match how_smooth {
        Smooth::Count(_) => Smooth::Count(0),
        Smooth::Continuous(_) => Smooth::Continuous(0.0),
    };
    let mut dist_traveled_next = dist_traveled_prev;
    let mut cur_next = &g.nodes[change.id];
    let mut cur_prev = &g.nodes[change.id];

    let how_smooth_f64 = how_smooth.as_f64();
    loop {
        cur_next = cur_next.next(g);
        cur_prev = cur_prev.prev(g);

        dist_traveled_next = dist_traveled_next.add(distance_between_nodes(&g.nodes[change.id], cur_next));
        dist_traveled_prev = dist_traveled_prev.add(distance_between_nodes(&g.nodes[change.id], cur_prev));

        let enough_next = dist_traveled_next.as_f64() > how_smooth_f64;
        let enough_prev = dist_traveled_prev.as_f64() > how_smooth_f64;

        if !enough_next {
            ret.insert(cur_next.id, mk_change(&cur_next, change, how_smooth_f64, dist_traveled_next.as_f64()));
        }
        if !enough_prev {
            ret.insert(cur_prev.id, mk_change(&cur_prev, change, how_smooth_f64, dist_traveled_prev.as_f64()));
        }
        if enough_next && enough_prev {
            break;
        }
    }
    ret
}

pub fn add_node_(ts: &mut ThickSurface, layer_to_which_add: usize, node_addition: NodeAddition) {
    ts.layers[layer_to_which_add].nodes[node_addition.n.next_id].prev_id = node_addition.n.id;
    ts.layers[layer_to_which_add].nodes[node_addition.n.prev_id].next_id = node_addition.n.id;
    ts.layers[layer_to_which_add].nodes.insert(
        node_addition.n.id,
        Node {
            id: node_addition.n.id,
            ..node_addition.n
        },
    );
}

pub fn delete_node_(ts: &mut ThickSurface, layer_from_which_delete: usize, node: Node) {
    /* 1. Remove node from the graph's circular path */
    let next_id = node.next_id;
    let prev_id = node.prev_id;
    ts.layers[layer_from_which_delete].nodes[prev_id].next_id = next_id;
    ts.layers[layer_from_which_delete].nodes[next_id].prev_id = prev_id;

    /* 2. Swap last node and deleted node's position */
    let last = ts.layers[layer_from_which_delete].nodes.last().unwrap().clone();
    let deleted_id = node.id;
    ts.layers[layer_from_which_delete].nodes[last.prev_id].next_id = deleted_id;
    ts.layers[layer_from_which_delete].nodes[last.next_id].prev_id = deleted_id;
    ts.layers[layer_from_which_delete].nodes[deleted_id] = last;
    ts.layers[layer_from_which_delete].nodes[deleted_id].id = deleted_id;

    /* 3. Shrink vector by 1 */
    let s = ts.layers[layer_from_which_delete].nodes.len();
    ts.layers[layer_from_which_delete].nodes.truncate(s - 1);
}

fn direction_vector0(_other_graph: &Graph, change: &NodeChange, _other_graph_changes: &NodeChangeMap) -> (f64, f64) {
    (change.cur_x, change.cur_y)
}

fn direction_vector1(other_graph: &Graph, change: &NodeChange, other_graph_changes: &NodeChangeMap) -> (f64, f64) {
    let changed_nodes_prev = other_graph.nodes[change.id].prev(other_graph);
    let (prev_ref_x, prev_ref_y) = match other_graph_changes.get(&changed_nodes_prev.id) {
        Some(nc) => (nc.cur_x + nc.delta_x, nc.cur_y + nc.delta_y),
        None => (changed_nodes_prev.x, changed_nodes_prev.y),
    };
    let changed_nodes_next = other_graph.nodes[change.id].next(other_graph);
    let (next_ref_x, next_ref_y) = match other_graph_changes.get(&changed_nodes_next.id) {
        Some(nc) => (nc.cur_x + nc.delta_x, nc.cur_y + nc.delta_y),
        None => (changed_nodes_next.x, changed_nodes_next.y),
    };
    /* prev_ref_xy and next_ref_xy are the position along which we want to find the direction vector */
    let (dir_x, dir_y) = bisecting_vector(change.cur_x, change.cur_y, prev_ref_x, prev_ref_y, next_ref_x, next_ref_y);

    (
        -dir_x * norm(change.delta_x, change.delta_y),
        -dir_y * norm(change.delta_x, change.delta_y),
    )
}

fn direction_from(org: (f64, f64), dst: (f64, f64)) -> (f64, f64) {
    let dist_v = (dst.0 - org.0, dst.1 - org.1);
    let norm = norm(dist_v.0, dist_v.1);
    (dist_v.0 / norm, dist_v.1 / norm)
}

fn would_change_intersect(index: usize, graph: &Graph, other_graph: &Graph, other_change: &NodeChange) -> bool {
    let (other_prev_x, other_prev_y) = other_graph.nodes[other_change.id].prev(other_graph).pos();
    let (other_next_x, other_next_y) = other_graph.nodes[other_change.id].next(other_graph).pos();
    let (other_changed_pos_x, other_changed_pos_y) = other_change.changed_pos();
    let (this_prev_x, this_prev_y) = graph.nodes[index].prev(graph).pos();
    let (this_next_x, this_next_y) = graph.nodes[index].next(graph).pos();
    let (this_pos_x, this_pos_y) = graph.nodes[index].pos();
    match lines_intersection(&vec![
        (other_prev_x, other_prev_y, other_changed_pos_x, other_changed_pos_y),
        (other_next_x, other_next_y, other_changed_pos_x, other_changed_pos_y),
        (this_prev_x, this_prev_y, this_pos_x, this_pos_y),
        (this_next_x, this_next_y, this_pos_x, this_pos_y),
    ]) {
        Some(_) => true,
        None => false,
    }
}

fn is_change_push2(index: usize, graph: &Graph, other_graph: &Graph, other_change: &NodeChange) -> bool {
    let (other_cs_next_x, other_cs_next_y) = other_graph.next(other_change.id).pos();
    let (other_cs_prev_x, other_cs_prev_y) = other_graph.prev(other_change.id).pos();
    let (pre_change_bisecting_x, pre_change_bisecting_y) = bisecting_vector(
        other_change.cur_x,
        other_change.cur_y,
        other_cs_next_x,
        other_cs_next_y,
        other_cs_prev_x,
        other_cs_prev_y,
    );
    println!("pre_change: {}, {}", other_change.cur_x, other_change.cur_y);
    println!("bisecting: {}, {}", pre_change_bisecting_x, pre_change_bisecting_y);
    println!(
        "post_change: {}, {}\n",
        other_change.cur_x + other_change.delta_x,
        other_change.cur_y + other_change.delta_x
    );

    let dist_cur_to_bisecting = distance_between_points(other_change.cur_x, other_change.cur_y, pre_change_bisecting_x, pre_change_bisecting_y);
    let dist_changed_to_bisecting = distance_between_points(
        other_change.changed_pos().0,
        other_change.changed_pos().1,
        pre_change_bisecting_x,
        pre_change_bisecting_y,
    );
    let dist_cur_to_inner = distance_between_points(other_change.cur_x, other_change.cur_y, graph.nodes[index].x, graph.nodes[index].y);
    dist_cur_to_bisecting > dist_changed_to_bisecting + dist_cur_to_inner
}

fn for_a_node_affected_make_the(index: usize, graph: &Graph, other_graph: &Graph, other_change: &NodeChange) -> NodeChange {
    let (potentially_wrongly_signed_dir_x, potentially_wrongly_signed_dir_y) = direction_from(
        (graph.nodes[index].x, graph.nodes[index].y),
        (other_change.cur_x + other_change.delta_x, other_change.cur_y + other_change.delta_y),
    );
    let (direction_x, direction_y) = if is_change_push2(index, graph, other_graph, other_change) {
        (-potentially_wrongly_signed_dir_x, -potentially_wrongly_signed_dir_y)
    } else {
        (potentially_wrongly_signed_dir_x, potentially_wrongly_signed_dir_y)
    };
    let distance_to_original_position = norm(graph.nodes[index].x - other_change.cur_x, graph.nodes[index].y - other_change.cur_y);

    let (desired_delta_x, desired_delta_y) = (direction_x * distance_to_original_position, direction_y * distance_to_original_position);
    let (changed_nodes_new_pos_x, changed_nodes_new_pos_y) = (other_change.cur_x + other_change.delta_x, other_change.cur_y + other_change.delta_y);
    let (desired_pos_x, desired_pos_y) = (changed_nodes_new_pos_x - desired_delta_x, changed_nodes_new_pos_y - desired_delta_y);
    let (delta_from_current_node_to_desired_pos_x, delta_from_current_node_to_desired_pos_y) =
        (desired_pos_x - graph.nodes[index].x, desired_pos_y - graph.nodes[index].y);

    NodeChange {
        id: index,
        cur_x: graph.nodes[index].x,
        cur_y: graph.nodes[index].y,
        delta_x: delta_from_current_node_to_desired_pos_x,
        delta_y: delta_from_current_node_to_desired_pos_y,
    }
}

fn weigh_change(inner_node_affected: usize, inner_graph: &Graph, outer_graph: &Graph, s: &Stitching, outer_change: &NodeChange) -> f64 {
    let outer_others = s.get(INNER, &inner_graph.nodes[inner_node_affected]);
    let (inn_x, inn_y) = inner_graph.nodes[inner_node_affected].pos();
    let total_distance = outer_others.iter().fold(0.0, |acc, x| {
        let (out_x, out_y) = outer_graph.nodes[*x].pos();
        acc + distance_between_points(inn_x, inn_y, out_x, out_y)
    });
    let distance_to_changed = distance_between_points(inn_x, inn_y, outer_change.cur_x, outer_change.cur_y);
    distance_to_changed / total_distance
}

fn for_ALL_nodes_affected_make_the(
    inner_nodes_affected: &Vec1<usize>,
    inner_graph: &Graph,
    outer_graph: &Graph,
    outer_change: &NodeChange,
    s: &Stitching,
) -> NodeChangeMap {
    let mut ret = NodeChangeMap::new();
    for i in inner_nodes_affected {
        let weight = weigh_change(*i, inner_graph, outer_graph, s, outer_change);
        println!("{}", weight);
        let change_individual = for_a_node_affected_make_the(*i, inner_graph, outer_graph, outer_change);
        ret.insert(
            *i,
            NodeChange {
                delta_x: change_individual.delta_x * weight,
                delta_y: change_individual.delta_y * weight,
                ..change_individual
            },
        );
    }
    ret
}

fn for_ALL_nodes_affected_make_the2(
    inner_nodes_affected: &Vec1<usize>,
    inner_graph: &Graph,
    outer_graph: &Graph,
    outer_change: &NodeChange,
    s: &Stitching,
) -> NodeChangeMap {
    let mut ret = NodeChangeMap::new();
    for i in inner_nodes_affected {
        let weight = weigh_change(*i, inner_graph, outer_graph, s, outer_change);
        let (cur_x, cur_y) = inner_graph.nodes[*i].pos();
        ret.insert(
            *i,
            NodeChange {
                id: *i,
                cur_x,
                cur_y,
                delta_x: outer_change.delta_x * weight,
                delta_y: outer_change.delta_y * weight,
            },
        );
    }
    ret
}

fn changes_from_other_graph(
    this_graph: &Graph,
    other_graph: &Graph,
    other_graph_changes: &NodeChangeMap,
    _compression_factor: f64,
    s: &Stitching,
) -> NodeChangeMap {
    let mut ret = NodeChangeMap::new();
    for (_, c) in other_graph_changes {
        let closest_node_affected = s.get_closest_correspondent(OUTER, &other_graph.nodes[c.id]);
        let inner_change = for_a_node_affected_make_the(closest_node_affected, this_graph, other_graph, c);
        ret.insert(inner_change.id, inner_change);
    }
    ret
}

fn changes_from_other_graph2(
    inner_graph: &Graph,
    outer_graph: &Graph,
    other_graph_changes: &NodeChangeMap,
    _compression_factor: f64,
    s: &Stitching,
) -> NodeChangeMap {
    let mut ret = NodeChangeMap::new();
    for (_, c) in other_graph_changes {
        let inner_nodes = s.get(OUTER, &outer_graph.nodes[c.id]);
        let inner_changes = for_ALL_nodes_affected_make_the(&inner_nodes, inner_graph, outer_graph, c, s);

        for (x, y) in &inner_changes {
            ret.insert(*x, y.clone());
        }
    }
    ret
}

fn changes_from_other_graph3(
    inner_graph: &Graph,
    outer_graph: &Graph,
    other_graph_changes: &NodeChangeMap,
    _compression_factor: f64,
    s: &Stitching,
) -> NodeChangeMap {
    let mut ret = NodeChangeMap::new();
    for (_, c) in other_graph_changes {
        let inner_nodes = s.get(OUTER, &outer_graph.nodes[c.id]);
        let inner_changes = for_ALL_nodes_affected_make_the2(&inner_nodes, inner_graph, outer_graph, c, s);

        for (x, y) in &inner_changes {
            ret.insert(*x, y.clone());
        }
    }
    ret
}

pub fn changer_of_choice(
    inner_graph: &Graph,
    outer_graph: &Graph,
    other_graph_changes: &NodeChangeMap,
    compression_factor: f64,
    s: &Stitching,
) -> NodeChangeMap {
    changes_from_other_graph(inner_graph, outer_graph, other_graph_changes, compression_factor, s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph::{area, circular_graph, circular_thick_surface, node_to_add};

    fn assert_cyclicness(g: &Graph) {
        let fst = &g.nodes[0];
        let mut j = fst.next(&g);
        let mut c = 0;
        loop {
            j = j.next(&g);
            if j == fst {
                break;
            }
            c = c + 1;
            if c >= g.nodes.len() {
                panic!("Looping too much forwards in assert_cyclicness")
            }
        }
        c = 0;
        loop {
            j = j.prev(&g);
            if j == fst {
                break;
            }
            c = c + 1;
            if c >= g.nodes.len() {
                panic!("Looping too much backwards in assert_cyclicness")
            }
        }
    }

    #[test]
    fn change_is_applied_and_reversed() {
        // TODO: This should be generated
        let size_of_test_circ = 40;

        let mut test_circ = circular_graph(0.0, 0.0, 1.0, size_of_test_circ);
        let area_before = area(&test_circ);
        let change = NodeChange {
            id: 1,
            cur_x: test_circ.nodes[1].x,
            cur_y: test_circ.nodes[1].y,
            delta_x: 70.0,
            delta_y: 100.0,
        };

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
        let to_add = node_to_add(
            &circular.layers[OUTER],
            &circular.layers[OUTER].nodes[10],
            &circular.layers[OUTER].nodes[10].next(&circular.layers[OUTER]),
            0.000001,
        );
        add_node_(&mut circular, OUTER, to_add.unwrap());
        let to_add = node_to_add(
            &circular.layers[OUTER],
            &circular.layers[OUTER].nodes[10],
            &circular.layers[OUTER].nodes[10].next(&circular.layers[OUTER]),
            0.000001,
        );
        add_node_(&mut circular, OUTER, to_add.unwrap());
        let to_add = node_to_add(
            &circular.layers[OUTER],
            &circular.layers[OUTER].nodes[10],
            &circular.layers[OUTER].nodes[10].next(&circular.layers[OUTER]),
            0.000001,
        );
        add_node_(&mut circular, OUTER, to_add.unwrap());

        let to_add = node_to_add(
            &circular.layers[INNER],
            &circular.layers[INNER].nodes[10],
            &circular.layers[INNER].nodes[10].next(&circular.layers[INNER]),
            0.000001,
        );
        add_node_(&mut circular, INNER, to_add.unwrap());

        /*TODO: FUCK, gotta fix these tests bad. */
    }
}
