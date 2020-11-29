use types::{ThickSurface, NodeChange, OUTER, INNER};
use graph_change::{apply_changes, revert_changes, random_change, smooth_change_out2, changes_from_other_graph, add_node_};
use graph;
use vector_2d_helpers::{lines_intersection};
use rand::Rng;
use std::collections::HashMap;

const SOME_HUGE_FUCKIN_VALUE: f64 = 100_000_000.0;

pub fn debug_changes(ts: &ThickSurface, how_smooth: usize, compression_factor: f64, which_node: usize, (x_change, y_change): (f64, f64)) -> (HashMap<usize, NodeChange>, HashMap<usize, NodeChange>) {
    let outer_change = NodeChange {
        id: which_node,
        cur_x: ts.layers[OUTER].nodes[which_node].x,
        cur_y: ts.layers[OUTER].nodes[which_node].y,
        delta_x: ts.layers[OUTER].nodes[which_node].x + x_change,
        delta_y: ts.layers[OUTER].nodes[which_node].y + y_change,
    };
    let smoothed_changes = smooth_change_out2(&ts.layers[OUTER], outer_change.clone(), how_smooth);
    let smoothed_inner_changes = changes_from_other_graph(&ts.layers[INNER], &ts.layers[OUTER], &smoothed_changes, compression_factor);
    (smoothed_changes, smoothed_inner_changes)
}

fn manual_neighbor_changes(ts: &ThickSurface,
                    node_change: NodeChange,
                    layer_to_push: usize,
                    layer_across: usize,
                    how_smooth: usize,
                    compression_factor: f64,
                    low_high: (f64, f64),
                    rng: &mut rand::rngs::ThreadRng) -> (HashMap<usize, NodeChange>, HashMap<usize, NodeChange>) {
    let smoothed_changes = smooth_change_out2(&ts.layers[layer_to_push], node_change.clone(), how_smooth);
    let smoothed_inner_changes = changes_from_other_graph(&ts.layers[layer_across], &ts.layers[layer_to_push], &smoothed_changes, compression_factor);
    (smoothed_changes, smoothed_inner_changes)
}

fn neighbor_changes(ts: &ThickSurface,
                    layer_to_push: usize,
                    layer_across: usize,
                    how_smooth: usize,
                    compression_factor: f64,
                    low_high: (f64, f64),
                    rng: &mut rand::rngs::ThreadRng) -> (HashMap<usize, NodeChange>, HashMap<usize, NodeChange>) {
    let outer_change = random_change(&ts.layers[layer_to_push], low_high, rng);
    let smoothed_changes = smooth_change_out2(&ts.layers[layer_to_push], outer_change.clone(), how_smooth);
    let smoothed_inner_changes = changes_from_other_graph(&ts.layers[layer_across], &ts.layers[layer_to_push], &smoothed_changes, compression_factor);
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
                        outer_changes: &HashMap<usize, NodeChange>,
                        inner_changes: &HashMap<usize, NodeChange>,
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

fn add_single_node_effects(ts: &mut ThickSurface, layer_to_add: usize, layer_across: usize, addition_threshold: f64) {
    let graph_to_which_add = &ts.layers[layer_to_add];
    let graph_across = &ts.layers[layer_across];

    for n in &graph_to_which_add.nodes {
        match graph::node_to_add(graph_to_which_add, graph_across, n, n.next(&graph_to_which_add), addition_threshold) {
            Some(addition) => {
                add_node_(ts, layer_to_add, layer_across, addition);
                break; // THE BREAK IS WHAT LETS THIS WORK, GODDAMN
            }
            None => {}
        }
    }
}

pub fn step(ts: &mut ThickSurface,
            initial_gray_matter_area: f64,
            temperature: f64,
            compression_factor: f64,
            how_smooth: usize,
            node_addition_threshold: f64,
            low_high: (f64, f64),
            rng: &mut rand::rngs::ThreadRng) {
    let (outer_changes, inner_changes) = neighbor_changes(ts, OUTER, INNER, how_smooth, compression_factor, low_high,rng);

    let energy_state = energy(ts, initial_gray_matter_area);
    apply_changes(&mut ts.layers[OUTER], &outer_changes);
    apply_changes(&mut ts.layers[INNER], &inner_changes);
    let energy_neighbor = energy(ts, initial_gray_matter_area);

    intersection_effects(ts, &outer_changes, &inner_changes, energy_state, energy_neighbor, temperature, rng);
    // add_single_node_effects(ts, OUTER, INNER, node_addition_threshold);
    add_single_node_effects(ts, INNER, OUTER, node_addition_threshold);
}

pub fn step_with_manual_change(ts: &mut ThickSurface,
            node_change: NodeChange,
            initial_gray_matter_area: f64,
            temperature: f64,
            compression_factor: f64,
            how_smooth: usize,
            node_addition_threshold: f64,
            low_high: (f64, f64),
            rng: &mut rand::rngs::ThreadRng) {
    let (outer_changes, inner_changes) = manual_neighbor_changes(ts, node_change, OUTER, INNER, how_smooth, compression_factor, low_high,rng);

    let energy_state = energy(ts, initial_gray_matter_area);
    apply_changes(&mut ts.layers[OUTER], &outer_changes);
    apply_changes(&mut ts.layers[INNER], &inner_changes);
    let energy_neighbor = energy(ts, initial_gray_matter_area);

    intersection_effects(ts, &outer_changes, &inner_changes, energy_state, energy_neighbor, temperature, rng);
    // add_single_node_effects(ts, OUTER, INNER, node_addition_threshold);
    add_single_node_effects(ts, INNER, OUTER, node_addition_threshold);
}
