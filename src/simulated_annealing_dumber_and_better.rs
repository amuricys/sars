use graph;
use graph::effects::{add_node_, apply_changes, changer_of_choice, merge_nodes_, random_change, smooth_change_out};
use graph::types::{Smooth, ThickSurface, INNER, OUTER};
use linalg_helpers::lines_intersection;
use rand::Rng;
use simulated_annealing::SimState;
use stitcher::types::Stitching;
use types::Params;

pub const PRACTICALLY_INFINITY: f64 = 100_000_000.0;

fn neighbor(
    ts: &ThickSurface,
    layer_to_push: usize,
    layer_across: usize,
    how_smooth: usize,
    compression_factor: f64,
    stitch: &Stitching,
    low_high: (f64, f64),
    addition_threshold: f64,
    deletion_threshold: f64,
    max_merge_steps_away: usize,
    rng: &mut rand::rngs::ThreadRng,
) -> ThickSurface {
    let mut ret = ts.clone();
    let outer_change = random_change(&ret.layers[layer_to_push], low_high, rng);
    let smoothed_changes = smooth_change_out(&ret.layers[layer_to_push], outer_change.clone(), Smooth::Count(how_smooth));
    let smoothed_inner_changes = changer_of_choice(
        &ret.layers[layer_across],
        &ret.layers[layer_to_push],
        &smoothed_changes,
        compression_factor,
        stitch,
    );
    apply_changes(&mut ret.layers[layer_to_push], &smoothed_changes);
    apply_changes(&mut ret.layers[layer_across], &smoothed_inner_changes);

    add_single_node_effects(&mut ret, layer_to_push, addition_threshold);
    add_single_node_effects(&mut ret, layer_across, addition_threshold);

    delete_single_node_effects(&mut ret, layer_to_push, deletion_threshold, max_merge_steps_away);
    delete_single_node_effects(&mut ret, layer_across, deletion_threshold, max_merge_steps_away);

    ret
}

pub fn energy(ts: &ThickSurface, initial_gray_matter_area: f64) -> f64 {
    let white_matter = graph::area(&ts.layers[INNER]);
    let gray_matter = (graph::area(&ts.layers[OUTER]) - white_matter).abs();
    let gray_matter_stretch = (gray_matter - initial_gray_matter_area).abs();

    // TODO: parametrize?
    white_matter + (1.0 + gray_matter_stretch).powf(2.0)
}

pub fn temperature(sim_state: &SimState, slope: f64) -> f64 {
    let new = sim_state.timestep as f64 * slope;
    if new < 0.0 {
        0.0
    } else {
        new
    }
}

fn probability_to_accept_neighbor_state(energy_state: f64, energy_neighbor: f64, temperature: f64) -> f64 {
    if temperature < 0.0 {
        if energy_neighbor < energy_state {
            1.0
        } else {
            0.0
        }
    } else if temperature >= PRACTICALLY_INFINITY {
        1.0
    } else {
        ((energy_state - energy_neighbor) / temperature).exp()
    }
}

fn should_move_to_neighbor(ts: &ThickSurface, energy_state: f64, energy_neighbor: f64, temperature: f64, rng: &mut rand::rngs::ThreadRng) -> bool {
    let lines1 = graph::graphs_to_lines(&ts.layers);
    let coin_flip = rng.gen_range(0.0, 1.0);
    match lines_intersection(&lines1) {
        Some(_) => false,
        None => {
            if probability_to_accept_neighbor_state(energy_state, energy_neighbor, temperature) < coin_flip {
                false
            } else {
                true
            }
        }
    }
}

fn add_single_node_effects(ts: &mut ThickSurface, layer_to_add: usize, addition_threshold: f64) {
    let graph_to_which_add = &ts.layers[layer_to_add];

    for n in &graph_to_which_add.nodes {
        match graph::node_to_add(graph_to_which_add, n, n.next(&graph_to_which_add), addition_threshold) {
            Some(addition) => {
                add_node_(ts, layer_to_add, &addition);
                break; // THE BREAK IS WHAT LETS THIS WORK, GODDAMN
            }
            None => {}
        }
    }
}

fn delete_single_node_effects(ts: &mut ThickSurface, layer_from_which_delete: usize, deletion_threshold: f64, max_merge_steps_away: usize) {
    let graph_from_which_delete = &ts.layers[layer_from_which_delete];
    for n in &graph_from_which_delete.nodes {
        match graph::nodes_to_merge(ts, layer_from_which_delete, n, deletion_threshold, max_merge_steps_away, false) {
            Some(deletion) => {
                merge_nodes_(ts, &deletion);
                break; // THE BREAK IS WHAT LETS THIS WORK, GODDAMN
            }
            None => {}
        }
    }
}

pub fn step(sim_state: &mut SimState, params: &Params) {
    let energy_state = energy(&sim_state.ts, params.initial_gray_matter_area);
    let neighbor = neighbor(
        &sim_state.ts,
        OUTER,
        INNER,
        params.how_smooth,
        params.compression_factor,
        &sim_state.stitching,
        params.low_high,
        params.node_addition_threshold,
        params.node_deletion_threshold,
        params.max_merge_steps_away,
        &mut sim_state.rng,
    );
    let energy_neighbor = energy(&neighbor, params.initial_gray_matter_area);

    if should_move_to_neighbor(&neighbor, energy_state, energy_neighbor, sim_state.temperature, &mut sim_state.rng) {
        sim_state.ts = neighbor;
    };

    sim_state.temperature = temperature(sim_state, params.temperature_param);
    sim_state.timestep += 1;
}
