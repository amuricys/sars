use std::f64::consts::PI;

use types::*;
use vec1::Vec1;
use vector_2d_helpers::{dist, norm};

pub fn cyclic_graph_from_coords(node_coordinates: &Vec1<(f64, f64)>) -> Graph {
    let mut to_return: Graph = Graph { nodes: Vec::new() };
    let _will_get_overridden_by_establish_corrs = 300;
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
    let mut circular_coords = Vec1::new((center_x + radius, center_y));
    for i in 1..num_points {
        circular_coords.push((
            center_x + (i as f64 * (2.0 * PI) / num_points as f64).cos() * radius,
            center_y + (i as f64 * (2.0 * PI) / num_points as f64).sin() * radius,
        ))
    }
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

        ret = ret + norm(cur.x - next.x, cur.y - next.y);

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
            dist(n1.x, n1.y, some_point_x, some_point_y)
                .partial_cmp(&dist(n2.x, n2.y, some_point_x, some_point_y))
                .unwrap()
        })
        .unwrap()
}

pub fn thick_surface_to_lines(ts: &ThickSurface) -> Vec<(f64, f64, f64, f64)> {
    let mut outer_lines = graph_to_lines(&ts.layers[OUTER]);
    let mut inner_lines = graph_to_lines(&ts.layers[INNER]);
    outer_lines.append(&mut inner_lines);
    outer_lines
}

pub fn distance_between_points(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    norm(x1 - x2, y1 - y2)
}

pub fn distance_between_nodes(n1: &Node, n2: &Node) -> f64 {
    norm(n1.x - n2.x, n1.y - n2.y)
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

pub fn node_to_delete(g: &Graph, prev: &Node, next: &Node, deletion_threshold: f64) -> Option<Node> {
    if prev.next(g).id == next.id && next.prev(g).id == prev.id && /* Might be worth moving all conditions to a function */
        distance_between_nodes(prev, next) < deletion_threshold
    {
        Some(prev.clone())
    } else {
        None
    }
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
