pub mod effects;
pub mod types;
mod convex_hull;

use graph::effects::merge_nodes_;
use graph::types::*;
use linalg_helpers;
use linalg_helpers::{dist, lines_intersection};
use renderer::lines_from_thick_surface;
use std::f32::INFINITY;

pub fn cyclic_graph_from_coords(node_coordinates: &Vec<(f64, f64)>) -> Graph {
    let mut to_return: Graph = Graph { nodes: Vec::new() };
    let num_points = node_coordinates.len();
    to_return.nodes.push(Node {
        id: 0,
        x: node_coordinates[0].0,
        y: node_coordinates[0].1,
        next_id: 1,
        prev_id: num_points - 1,
    });
    for i in 1..num_points {
        let new_node = Node {
            id: i,
            x: node_coordinates[i].0,
            y: node_coordinates[i].1,
            next_id: (i + 1) % num_points,
            prev_id: i - 1,
        };
        to_return.nodes.push(new_node);
    }

    to_return
}

pub fn circular_graph(center_x: f64, center_y: f64, radius: f64, num_points: usize) -> Graph {
    let circular_coords = linalg_helpers::circular_points(center_x, center_y, radius, num_points);
    cyclic_graph_from_coords(&circular_coords)
}

pub fn circular_thick_surface(radius: f64, thickness: f64, num_points: usize) -> ThickSurface {
    let outer = circular_graph(0.0, 0.0, radius, num_points);
    let inner = circular_graph(0.0, 0.0, radius - thickness, num_points);
    ThickSurface {
        layers: Vec::from([outer, inner]),
    }
}

pub fn gray_matter_area(ts: &ThickSurface) -> f64 {
    area(&ts.layers[OUTER]) - area(&ts.layers[INNER])
}

pub fn area(g: &Graph) -> f64 {
    let mut ret = 0.0;
    for n in &g.nodes {
        let prev = n.prev(g);
        let next = n.next(g);

        ret = ret + n.x * (next.y - prev.y);
    }
    ret / 2.0
}

pub fn perimeter(g: &Graph) -> f64 {
    let mut ret = 0.0;
    let first = &g.nodes[0];
    let mut cur = first;
    loop {
        let next = cur.next(g);

        ret = ret + linalg_helpers::norm(cur.x - next.x, cur.y - next.y);

        cur = next;
        if cur == first {
            break;
        }
    }
    ret
}

fn graph_to_lines(g: &Graph) -> Vec<(f64, f64, f64, f64)> {
    let mut ret = Vec::new();
    for n in &g.nodes {
        let n_next = n.next(g);
        ret.push((n.x, n.y, n_next.x, n_next.y));
    }
    ret
}

pub fn closest_node_to_some_point(graph: &Graph, some_point_x: f64, some_point_y: f64) -> &Node {
    graph
        .nodes
        .iter()
        .min_by(|n1, n2| {
            linalg_helpers::dist(n1.x, n1.y, some_point_x, some_point_y)
                .partial_cmp(&linalg_helpers::dist(n2.x, n2.y, some_point_x, some_point_y))
                .unwrap()
        })
        .unwrap()
}

pub fn closest_nodes_to_some_point(graph: &Graph, some_point_x: f64, some_point_y: f64) -> (&Node, &Node) {
    let closest = closest_node_to_some_point(graph, some_point_x, some_point_y);
    if dist(closest.next(graph).x, closest.next(graph).y, some_point_x, some_point_y)
        < dist(closest.prev(graph).x, closest.prev(graph).y, some_point_x, some_point_y)
    {
        (closest, closest.next(graph))
    } else {
        (closest.prev(graph), closest)
    }
}

pub fn closest_node_across_all_layers(ts: &ThickSurface, some_point_x: f64, some_point_y: f64) -> (&Node, usize) {
    let (mut so_far, mut ret) = (f64::INFINITY, (&ts.layers[OUTER].nodes[0], 0));

    for l in 0..ts.layers.len() {
        let hmm = closest_node_to_some_point(&ts.layers[l], some_point_x, some_point_y);
        if dist(hmm.x, hmm.y, some_point_x, some_point_y) < so_far {
            so_far = dist(hmm.x, hmm.y, some_point_x, some_point_y);
            ret = (hmm, l);
        }
    }
    ret
}

pub fn closest_nodes_across_all_layers(ts: &ThickSurface, some_point_x: f64, some_point_y: f64) -> (&Node, &Node, usize) {
    let (mut so_far, mut ret) = (f64::INFINITY, (&ts.layers[OUTER].nodes[0], &ts.layers[OUTER].nodes[0], 0));

    for l in 0..ts.layers.len() {
        let (hmm_p, hmm_n) = closest_nodes_to_some_point(&ts.layers[l], some_point_x, some_point_y);
        if dist(hmm_p.x, hmm_p.y, some_point_x, some_point_y) < so_far {
            so_far = dist(hmm_p.x, hmm_p.y, some_point_x, some_point_y);
            ret = (hmm_p, hmm_n, l);
        }
    }
    ret
}

pub fn graphs_to_lines(graphs: &Vec<Graph>) -> Vec<(f64, f64, f64, f64)> {
    let mut ret = Vec::new();
    for g in graphs {
        let mut lines = graph_to_lines(g);
        ret.append(&mut lines);
    }
    ret
}

pub fn distance_between_points(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    linalg_helpers::norm(x1 - x2, y1 - y2)
}

pub fn distance_between_nodes(n1: &Node, n2: &Node) -> f64 {
    linalg_helpers::norm(n1.x - n2.x, n1.y - n2.y)
}

pub fn available_node_id(g: &Graph) -> usize {
    g.nodes.len()
}

pub fn node_to_add(g: &Graph, prev: &Node, next: &Node, addition_threshold: f64) -> Option<NodeAddition> {
    if prev.next(g).id == next.id && next.prev(g).id == prev.id && /* Might be worth moving all conditions to a function */
        distance_between_nodes(prev, next) > addition_threshold
    {
        let new_node_id = available_node_id(g);

        let new_node = Node {
            id: new_node_id,
            x: (prev.x + next.x) / 2.0,
            y: (prev.y + next.y) / 2.0,
            next_id: next.id,
            prev_id: prev.id,
        };
        Some(NodeAddition { n: new_node })
    } else {
        None
    }
}

fn merging_wouldnt_add_intersection(ts: &ThickSurface, node_merging: &NodeMerging) -> bool {
    let mut simulated_ts = ts.clone();
    merge_nodes_(&mut simulated_ts, node_merging);
    match lines_intersection(&graphs_to_lines(&simulated_ts.layers)) {
        Some(_) => false,
        None => true,
    }
}

fn can_merge(ts: &ThickSurface, node_merging: &NodeMerging, deletion_threshold: f64) -> bool {
    distance_between_nodes(&node_merging.one_end, &node_merging.oth_end) < deletion_threshold && merging_wouldnt_add_intersection(ts, node_merging)
}

fn can_merge_without_intersection_check(ts: &ThickSurface, node_merging: &NodeMerging, deletion_threshold: f64) -> bool {
    distance_between_nodes(&node_merging.one_end, &node_merging.oth_end) < deletion_threshold
}

#[derive(Clone, Debug)]
pub struct NodeMerging {
    one_end: Node,
    oth_end: Node,
    dist: usize,
    layer_id: usize,
    survivor_x: f64,
    survivor_y: f64,
}

pub fn nodes_to_merge(
    ts: &ThickSurface,
    layer_id: usize,
    src: &Node,
    deletion_threshold: f64,
    max_merge_steps_away: usize,
    check_ints: bool,
) -> Option<NodeMerging> {
    for i in 1..max_merge_steps_away + 1 {
        let nnnn = src.clone();
        let mmmm = src.next_by(&ts.layers[layer_id], i).clone();
        let (avg_x, avg_y) = ((nnnn.x + mmmm.x) / 2.0, (nnnn.y + mmmm.y) / 2.0);
        let m = NodeMerging {
            one_end: nnnn,
            oth_end: mmmm,
            dist: i,
            layer_id: layer_id,
            survivor_x: avg_x,
            survivor_y: avg_y,
        };
        let b = match check_ints {
            true => can_merge(ts, &m, deletion_threshold),
            false => can_merge_without_intersection_check(ts, &m, deletion_threshold),
        };
        if b {
            return Some(m);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_go_around() {
        // TODO: This should be generated
        let size_of_test_circ = 4;

        let test_circ = circular_graph(0.0, 0.0, 1.0, size_of_test_circ);
        let mut walker = &test_circ.nodes[0];
        let first = test_circ.nodes[0].clone();
        for _i in 0..size_of_test_circ {
            walker = walker.next(&test_circ);
        }
        assert_eq!(*walker, first);
    }

    #[test]
    fn circular_area() {
        let size_of_graph = 200;
        let test_circ = circular_graph(0.0, 0.0, 1.0, size_of_graph);

        assert!(area(&test_circ) < 3.15);
        assert!(area(&test_circ) > 3.13);
    }

    #[test]
    fn circular_perimeter() {
        let size_of_graph = 200;
        let test_circ = circular_graph(2.0, 7.0, 1.0, size_of_graph);

        assert!(perimeter(&test_circ) < 6.30);
        assert!(perimeter(&test_circ) > 6.26);
    }
}
