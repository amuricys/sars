use std::f64::consts::PI;

use types::*;
use vector_2d_helpers::{norm};
use vec1::Vec1;

pub fn bleh_graph(num_points: usize, is_outer: bool) -> Graph {
    let mut to_return: Graph = Graph { nodes: vec![], edges: vec![] };
    let def_acrossness = Acrossness {
        mid: None,
        prev: None,
        next: None
    };
    for i in 0..num_points-2 {
        let new_edge = EdgeSameSurface{source: if i == 0 {999} else {i-1}, target: i, id: if i == 0 {999} else {i-1}};
        to_return.edges.push(new_edge);

        let new_node = Node {
            id: i,
            x: 1.0 - i as f64 / num_points as f64,
            y: if is_outer {0.2} else {-0.2},
            inc: if i == 0 {999} else {i-1},
            out: i,
            acrossness: def_acrossness};
        to_return.nodes.push(new_node);
    }
    to_return.nodes.push(Node {
        id: num_points-1,
        x: -1.0,
        y: if is_outer {0.2} else {-0.2},
        inc: num_points-1,
        out: 999,
        acrossness: def_acrossness});
    to_return
}

pub fn circular_graph(center_x: f64, center_y: f64, radius: f64, num_points: usize) -> Graph {
    let mut to_return: Graph = Graph { nodes: vec![], edges: vec![] };
    let def_acrossness = Acrossness {
        mid: None,
        prev: None,
        next: None
    };

    to_return.nodes.push(Node{id: 0, x: center_x + radius, y: center_y as f64, inc: num_points-1, out: 0, acrossness: def_acrossness});
    for i in 1..num_points {
        let new_edge = EdgeSameSurface{source: i-1, target: i, id: i-1};
        to_return.edges.push(new_edge);

        let new_node = Node{id: i,
            x: center_x as f64 + (i as f64 * (2.0 * PI) / num_points as f64).cos() * radius,
            y: center_y as f64 + (i as f64 * (2.0 * PI) / num_points as f64).sin() * radius,
            inc: i-1,
            out: i,
            acrossness: def_acrossness
        };
        to_return.nodes.push(new_node);
    }
    let new_edge = EdgeSameSurface{source: num_points - 1, target: 0, id: num_points - 1};
    to_return.edges.push(new_edge);

    to_return
}

fn establish_correspondences(outer: &mut Graph, inner: &mut Graph) {
    for i in 0..outer.nodes.len() {
        outer.nodes[i].acrossness.mid = Some(i);
        inner.nodes[i].acrossness.mid = Some(i);
    }
}

pub fn debug_straight_surface(num_points: usize) -> ThickSurface {
    let mut outer = bleh_graph(num_points, true);
    let mut inner = bleh_graph(num_points, false);
    establish_correspondences(&mut outer, &mut inner);
    ThickSurface{layers: Vec::from([outer, inner]), edges: Vec::new()}
}

pub fn circular_thick_surface(radius: f64, thickness: f64, num_points: usize) -> ThickSurface {
    let mut outer = circular_graph(0.0, 0.0, radius, num_points);
    let mut inner = circular_graph(0.0, 0.0, radius - thickness, num_points);
    establish_correspondences(&mut outer, &mut inner);
    ThickSurface{layers: Vec::from([outer, inner]), edges: Vec::new()}
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
    let mut outer_lines = graph_to_lines(&ts.layers[OUTER]);
    let mut inner_lines = graph_to_lines(&ts.layers[INNER]);
    outer_lines.append(&mut inner_lines);
    outer_lines
}

pub fn distance_between_nodes(n1: &Node, n2: &Node) -> f64 {
    norm(n1.x - n2.x, n1.y - n2.y)
}

fn available_node_id(g: &Graph) -> usize {
    /* Graph nodes should be Option(Node)s */
    g.nodes.len()
}

fn find_some_fucking_shit_idk_NAMMEESSS() {

}

pub fn node_to_add(g: &Graph, prev: &Node, next: &Node, addition_threshold: f64) -> Option<NodeAddition> {
    let new_node_id = available_node_id(g);
    if prev.next(g).id == next.id && next.prev(g).id == prev.id && /* Might be worth moving all conditions to a function */
       distance_between_nodes(prev, next) > addition_threshold {
        let acrossness = find_some_fucking_shit_idk_NAMMEESSS();
        Some(NodeAddition {
            id: new_node_id,
            prev_id: prev.id,
            next_id: next.id,
            acrossness: Vec1::new(1)
        })
    } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simulated_annealing::debug_changes;

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

        let test_circ = circular_thick_surface(1.0, 0.3, size_of_test_circ);
        for n in &test_circ.layers[OUTER].nodes {
            assert_eq!(n.acrossness.mid.unwrap(), n.id)
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
