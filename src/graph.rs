use std::collections::HashSet;
use std::f64::consts::PI;

use types::*;
use vector_2d_helpers::{norm};


pub fn circular_graph(center_x: f64, center_y: f64, radius: f64, num_points: usize) -> Graph {
    let mut to_return: Graph = Graph { nodes: vec![], edges: vec![] };

    to_return.nodes.push(Node{id: 0, x: center_x + radius, y: center_y as f64, inc: num_points-1, out: 0, across: HashSet::new()});
    for i in 1..num_points {
        let new_edge = EdgeSameSurface{source: i-1, target: i, id: i-1};
        to_return.edges.push(new_edge);

        let new_node = Node{id: i,
            x: center_x as f64 + (i as f64 * (2.0 * PI) / num_points as f64).cos() * radius,
            y: center_y as f64 + (i as f64 * (2.0 * PI) / num_points as f64).sin() * radius,
            inc: i-1,
            out: i,
            across: HashSet::new()};
        to_return.nodes.push(new_node);
    }
    let new_edge = EdgeSameSurface{source: num_points - 1, target: 0, id: num_points - 1};
    to_return.edges.push(new_edge);

    to_return
}

fn establish_correspondences(outer: &mut Graph, inner: &mut Graph) -> ThickSurface {
    for i in 0..outer.nodes.len() {
        outer.nodes[i].across.insert(i);
        inner.nodes[i].across.insert(i);
    }
    ThickSurface {outer: outer.clone(), inner: inner.clone(), edges: vec![] }
}

pub fn thick_surface(radius: f64, thickness: f64, num_points: usize) -> ThickSurface {
    let mut outer = circular_graph(0.0, 0.0, radius, num_points);
    let mut inner = circular_graph(0.0, 0.0, radius - thickness, num_points);

    establish_correspondences(&mut outer, &mut inner)
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
        if cur == first { break; }
    }
    ret
}

fn graph_to_lines(g: &Graph) -> Vec<(f64,f64,f64,f64)> {
    let mut ret = Vec::new();
    for edge in &g.edges {
        let node1 = &g.nodes[edge.source];
        let node2 = &g.nodes[edge.target];
        ret.push((node1.x, node1.y, node2.x, node2.y));
    }
    ret
}

pub fn thick_surface_to_lines(ts: &ThickSurface) -> Vec<(f64,f64,f64,f64)> {
    let mut outer_lines = graph_to_lines(&ts.outer);
    let mut inner_lines = graph_to_lines(&ts.inner);
    outer_lines.append(&mut inner_lines);
    outer_lines
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
    fn correspondences() {
        // TODO: This should be generated
        let size_of_test_circ = 4;

        let test_circ = thick_surface(1.0, 0.3, size_of_test_circ);
        for n in test_circ.outer.nodes {
            assert!(n.across.contains(&n.id))
        }
    }

    #[test]
    fn circular_area() {
        let size_of_graph = 200;
        let test_circ = circular_graph(0.0, 0.0, 1.0, size_of_graph);

        println!("area: {:?}", area(&test_circ));

        assert!(area(&test_circ) < 3.15);
        assert!(area(&test_circ) > 3.13);

    }

    #[test]
    fn circular_perimeter() {
        let size_of_graph = 200;
        let test_circ = circular_graph(2.0, 7.0, 1.0, size_of_graph);

        println!("{:?}", perimeter(&test_circ));

        assert!(perimeter(&test_circ) < 6.30);
        assert!(perimeter(&test_circ) > 6.26);
    }
}
