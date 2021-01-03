use std::f64::consts::PI;

use types::*;
use vector_2d_helpers::{norm};
use vec1::Vec1;
use std::collections::HashMap;

pub fn cyclic_graph_from_coords(node_coordinates: &Vec1<(f64, f64)>) -> Graph {
    let mut to_return: Graph = Graph { nodes: HashMap::new() };
    let will_get_overridden_by_establish_corrs = 300;
    let num_points = node_coordinates.len();
    to_return.nodes.insert(0, Node{id: 0, x: node_coordinates[0].0, y: node_coordinates[0].1, next_id: 1, prev_id: num_points-1, acrossness: Vec1::new(will_get_overridden_by_establish_corrs)});
    for i in 1..num_points {
        let new_node = Node{id: i,
            x: node_coordinates[i].0,
            y: node_coordinates[i].1,
            next_id: (i+1) % num_points,
            prev_id: i-1,
            acrossness: Vec1::new(will_get_overridden_by_establish_corrs)
        };
        to_return.nodes.insert(i, new_node);
    }

    to_return
}

pub fn circular_graph(center_x: f64, center_y: f64, radius: f64, num_points: usize) -> Graph {
    let mut circular_coords = Vec1::new((center_x + radius, center_y));
    for i in 1..num_points {
        circular_coords.push((
            center_x + (i as f64 * (2.0 * PI) / num_points as f64).cos() * radius,
            center_y + (i as f64 * (2.0 * PI) / num_points as f64).sin() * radius)
        )
    }
    cyclic_graph_from_coords(&circular_coords)
}

pub fn establish_correspondences(outer: &mut Graph, inner: &mut Graph) {
    for i in 0..outer.nodes.len() {
        outer.nodes.get_mut(&i).unwrap().acrossness[0] = i;
        inner.nodes.get_mut(&i).unwrap().acrossness[0] = i;
    }
}

pub fn debug_straight_surface(num_points: usize) -> ThickSurface {
    panic!("Will make it soon");
    // establish_correspondences(&mut outer, &mut inner);
    // ThickSurface{layers: Vec::from([outer, inner]), edges: Vec::new()}
}

pub fn circular_thick_surface(radius: f64, thickness: f64, num_points: usize) -> ThickSurface {
    let mut outer = circular_graph(0.0, 0.0, radius, num_points);
    let mut inner = circular_graph(0.0, 0.0, radius - thickness, num_points);
    establish_correspondences(&mut outer, &mut inner);
    ThickSurface{layers: Vec::from([outer, inner])}
}

pub fn gray_matter_area(ts: &ThickSurface) -> f64 {
    area(&ts.layers[OUTER]) - area(&ts.layers[INNER])
}

pub fn area(g: &Graph) -> f64 {
    let mut ret = 0.0;
    for (_, n) in &g.nodes {
        let prev = n.prev(g);
        let next = n.next(g);

        ret = ret + n.x * (next.y - prev.y);
    }
    ret / 2.0
}

pub fn perimeter(g: &Graph) -> f64 {
    let mut ret = 0.0;
    let (_, first) = g.nodes.iter().nth(0).unwrap();
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
    for (_, n) in &g.nodes {
        let n_next = n.next(g);
        ret.push((n.x, n.y, n_next.x, n_next.y));
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

pub fn available_node_id(g: &Graph) -> usize {
    /* Graph nodes should be Option(Node)s */
    for i in 0..g.nodes.len() {
        match g.nodes.get(&i) {
            None => return i,
            Some(_) => continue,
        }
    }
    g.nodes.len()
}

fn find_acrossness(g_across: &Graph, prev: &Node, next: &Node) -> Vec1<NodeIndex> {
    fn check_common_neighborhood(g: &Graph, n0: &usize, n1: &usize) -> bool {
        g.nodes.get(n0).unwrap().next(g).id == *n1 ||
        g.nodes.get(n0).unwrap().prev(g).id == *n1 ||
        g.nodes.get(n1).unwrap().prev(g).id == *n0 ||
        g.nodes.get(n1).unwrap().prev(g).id == *n0
    }
    for ac0 in &prev.acrossness {
        for ac1 in &next.acrossness {
            if ac0 == ac1 {
                return Vec1::new(*ac0)
            }
            if check_common_neighborhood(g_across, ac0, ac1) {
                let mut ret = Vec1::new(*ac0); ret.push(*ac1);
                return ret
            }
        }
    }
    for ac0 in &prev.acrossness {
        for ac1 in &next.acrossness {
            println!("Checking if {} and {} are equal...", ac0, ac1);
            println!("Checking if {} and {} have neighbors in common:", ac0, ac1);
            println!("\t{}'s next(): {:?}; prev(): {:?}", *ac0, g_across.nodes.get(ac0).unwrap().next(g_across).id, g_across.nodes.get(ac0).unwrap().prev(g_across).id);
            println!("\t{}'s next(): {:?}; prev(): {:?}", *ac1, g_across.nodes.get(ac1).unwrap().next(g_across).id, g_across.nodes.get(ac1).unwrap().prev(g_across).id);
        }
    }
    println!("prev's across: {:?}", prev.acrossness);
    println!("next's across: {:?}", next.acrossness);
    panic!("Could not find shit.")
}

pub fn node_to_add(g: &Graph, g_across: &Graph, prev: &Node, next: &Node, addition_threshold: f64) -> Option<NodeAddition> {
    if prev.next(g).id == next.id && next.prev(g).id == prev.id && /* Might be worth moving all conditions to a function */
       distance_between_nodes(prev, next) > addition_threshold {

        let new_node_id = available_node_id(g);

        let new_node = Node {
            id: new_node_id,
            x:  (prev.x + next.x) / 2.0,
            y: (prev.y + next.y) / 2.0,
            next_id: next.id,
            prev_id: prev.id,
            acrossness: find_acrossness(g_across, prev, next)};
        Some( NodeAddition { n: new_node  })
    } else { None }
}

pub fn node_to_delete(g: &Graph, prev: &Node, next: &Node, deletion_threshold: f64) -> Option<(NodeIndex, NodeIndex)> {
    if prev.next(g).id == next.id && next.prev(g).id == prev.id && /* Might be worth moving all conditions to a function */
        distance_between_nodes(prev, next) < deletion_threshold {
        Some((prev.id, next.id))
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
            assert_eq!(n.acrossness[0], n.id)
        }
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
