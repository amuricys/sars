use types::{ThickSurface, NodeChange, OUTER, INNER};
use graph_change::{apply_changes, revert_changes, random_change, smooth_change_out2, changes_from_other_graph, add_node_};
use graph;
use vector_2d_helpers::{lines_intersection};
use rand::Rng;
use piston::input::keyboard::Key::Out;

const SOME_HUGE_FUCKIN_VALUE: f64 = 100_000_000.0;

fn neighbor_changes(ts: &ThickSurface, how_smooth: usize, compression_factor: f64, rng: &mut rand::rngs::ThreadRng) -> (Vec<NodeChange>, Vec<NodeChange>) {
    let outer_change = random_change(&ts.layers[OUTER], (-0.2, 0.2), rng);
    let smoothed_changes = smooth_change_out2(&ts.layers[OUTER], outer_change.clone(), how_smooth);
    let smoothed_inner_changes = changes_from_other_graph(&ts.layers[INNER], &ts.layers[OUTER], &smoothed_changes, compression_factor);
    (smoothed_changes, smoothed_inner_changes)
}

fn energy(ts: &ThickSurface, initial_gray_matter_area: f64) -> f64 {
    let white_matter = graph::area(&ts.layers[INNER]);
    let gray_matter = (graph::area(&ts.layers[OUTER]) - white_matter).abs();
    let gray_matter_stretch = (gray_matter - initial_gray_matter_area).abs();

    // TODO: parametrize?
    white_matter + (1.0 + gray_matter_stretch).powf(2.0)
}

fn probability(energy_state: f64, energy_neighbor: f64, temperature: f64) -> f64 {
    if temperature < 0.0 {
        if energy_neighbor < energy_state { 1.0 } else { 0.0 }
    } else if temperature >= SOME_HUGE_FUCKIN_VALUE {
        1.0
    } else {
        ((energy_state - energy_neighbor) / temperature).exp()
    }
}

fn intersection_effects(ts: &mut ThickSurface,
                        outer_changes: &Vec<NodeChange>,
                        inner_changes: &Vec<NodeChange>,
                        energy_state: f64,
                        energy_neighbor: f64,
                        temperature: f64,
                        rng: &mut rand::rngs::ThreadRng) {
    let lines = graph::thick_surface_to_lines(ts);
    match lines_intersection(&lines) {
        Some(_) => {
            revert_changes(&mut ts.layers[OUTER], outer_changes);
            revert_changes(&mut ts.layers[INNER], inner_changes);
        }
        None => {
            let coin_flip = rng.gen_range(0.0, 1.0);
            if probability(energy_state, energy_neighbor, temperature) < coin_flip {
                revert_changes(&mut ts.layers[OUTER], outer_changes);
                revert_changes(&mut ts.layers[INNER], inner_changes);
            }
        }
    }
}

fn node_addition_effects(ts: &mut ThickSurface, addition_threshold: f64) {
    let mut nodes_to_add = Vec::new();
    let graph_to_which_add = &ts.layers[OUTER];
    for n in &graph_to_which_add.nodes {
        let optional_node = graph::node_to_add(&graph_to_which_add, n, n.next(&graph_to_which_add), addition_threshold);
        match optional_node{
            Some(nodeness) => nodes_to_add.push(nodeness),
            None => {}
        }
    }
    for nodeness in nodes_to_add {
        add_node_(ts, nodeness);
        println!("adding between {} and {}", nodeness.prev_id, nodeness.next_id);
    }
}

pub fn step(ts: &mut ThickSurface,
            initial_gray_matter_area: f64,
            temperature: f64,
            compression_factor: f64,
            how_smooth: usize,
            rng: &mut rand::rngs::ThreadRng) {
    let (outer_changes, inner_changes) = neighbor_changes(ts, how_smooth, compression_factor, rng);

    let energy_state = energy(ts, initial_gray_matter_area);
    apply_changes(&mut ts.layers[OUTER], &outer_changes);
    apply_changes(&mut ts.layers[INNER], &inner_changes);
    let energy_neighbor = energy(ts, initial_gray_matter_area);

    intersection_effects(ts, &outer_changes, &inner_changes, energy_state, energy_neighbor, temperature, rng);
    // node_addition_effects(&mut ts.layers[OUTER], 0.05);
    node_addition_effects(ts, 0.05);
}
