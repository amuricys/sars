use graph;
use graph::effects::{add_node_, apply_changes, changes_from_other_graph, delete_node_, random_change, revert_changes, smooth_change_out};
use graph::types::{NodeChange, NodeChangeMap, Smooth, ThickSurface, INNER, OUTER};
use linalg_helpers::lines_intersection;
use rand::Rng;
use stitcher::Stitching;
use types::Params;

const PRACTICALLY_INFINITY: f64 = 100_000_000.0;

fn manual_neighbor_changes(
    ts: &ThickSurface,
    node_change: NodeChange,
    layer_to_push: usize,
    layer_across: usize,
    how_smooth: usize,
    compression_factor: f64,
    stitch: Stitching,
) -> (NodeChangeMap, NodeChangeMap) {
    let smoothed_changes = smooth_change_out(&ts.layers[layer_to_push], node_change.clone(), Smooth::Count(how_smooth));
    let smoothed_inner_changes = changes_from_other_graph(
        &ts.layers[layer_across],
        &ts.layers[layer_to_push],
        &smoothed_changes,
        compression_factor,
        stitch,
    );
    (smoothed_changes, smoothed_inner_changes)
}

fn neighbor_changes(
    ts: &ThickSurface,
    layer_to_push: usize,
    layer_across: usize,
    how_smooth: usize,
    compression_factor: f64,
    stitch: Stitching,
    low_high: (f64, f64),
    rng: &mut rand::rngs::ThreadRng,
) -> (NodeChangeMap, NodeChangeMap) {
    let outer_change = random_change(&ts.layers[layer_to_push], low_high, rng);
    let smoothed_changes = smooth_change_out(&ts.layers[layer_to_push], outer_change.clone(), Smooth::Count(how_smooth));
    let smoothed_inner_changes = changes_from_other_graph(
        &ts.layers[layer_across],
        &ts.layers[layer_to_push],
        &smoothed_changes,
        compression_factor,
        stitch,
    );
    (smoothed_changes, smoothed_inner_changes)
}

pub fn energy(ts: &ThickSurface, initial_gray_matter_area: f64) -> f64 {
    let white_matter = graph::area(&ts.layers[INNER]);
    let gray_matter = (graph::area(&ts.layers[OUTER]) - white_matter).abs();
    let gray_matter_stretch = (gray_matter - initial_gray_matter_area).abs();

    // TODO: parametrize?
    white_matter + (1.0 + gray_matter_stretch).powf(2.0)
}

fn probability(energy_state: f64, energy_neighbor: f64, temperature: f64) -> f64 {
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

fn intersection_effects(
    ts: &mut ThickSurface,
    outer_changes: &NodeChangeMap,
    inner_changes: &NodeChangeMap,
    energy_state: f64,
    energy_neighbor: f64,
    temperature: f64,
    rng: &mut rand::rngs::ThreadRng,
) {
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

fn add_single_node_effects(ts: &mut ThickSurface, layer_to_add: usize, addition_threshold: f64) {
    let graph_to_which_add = &ts.layers[layer_to_add];

    for n in &graph_to_which_add.nodes {
        match graph::node_to_add(graph_to_which_add, n, n.next(&graph_to_which_add), addition_threshold) {
            Some(addition) => {
                println!("added");
                add_node_(ts, layer_to_add, addition);
                break; // THE BREAK IS WHAT LETS THIS WORK, GODDAMN
            }
            None => {}
        }
    }
}

fn delete_single_node_effects(ts: &mut ThickSurface, layer_from_which_delete: usize, deletion_threshold: f64) {
    let graph_from_which_delete = &ts.layers[layer_from_which_delete];

    for n in &graph_from_which_delete.nodes {
        match graph::node_to_delete(graph_from_which_delete, n, n.next(&graph_from_which_delete), deletion_threshold) {
            Some(deletion) => {
                println!("deleted");
                delete_node_(ts, layer_from_which_delete, deletion);
                break; // THE BREAK IS WHAT LETS THIS WORK, GODDAMN
            }
            None => {}
        }
    }
}

static mut THING: bool = false;

pub fn step(
    ts: &mut ThickSurface,
    initial_gray_matter_area: f64,
    temperature: f64,
    stitch: Stitching,
    params: &Params,
    rng: &mut rand::rngs::ThreadRng,
) -> Vec<NodeChangeMap> {
    let how_smooth = params.how_smooth;
    let compression_factor = params.compression_factor;
    let low_high = params.low_high;
    let node_addition_threshold = params.node_addition_threshold;
    let node_deletion_threshold = params.node_deletion_threshold;

    let (outer_changes, inner_changes) = neighbor_changes(ts, OUTER, INNER, how_smooth, compression_factor, stitch, low_high, rng);

    let energy_state = energy(ts, initial_gray_matter_area);
    apply_changes(&mut ts.layers[OUTER], &outer_changes);
    apply_changes(&mut ts.layers[INNER], &inner_changes);
    let energy_neighbor = energy(ts, initial_gray_matter_area);

    intersection_effects(ts, &outer_changes, &inner_changes, energy_state, energy_neighbor, temperature, rng);
    unsafe {
        if !THING {
            add_single_node_effects(ts, OUTER, node_addition_threshold);
            add_single_node_effects(ts, INNER, node_addition_threshold);
            THING = !THING;
        } else {
            delete_single_node_effects(ts, OUTER, node_deletion_threshold);
            delete_single_node_effects(ts, INNER, node_deletion_threshold);
            THING = !THING;
        }
    }
    vec![outer_changes, inner_changes]
}

pub fn step_with_manual_change(
    ts: &mut ThickSurface,
    node_change: NodeChange,
    initial_gray_matter_area: f64,
    temperature: f64,
    stitch: Stitching,
    params: &Params,
    rng: &mut rand::rngs::ThreadRng,
) -> Vec<NodeChangeMap> {
    let how_smooth = params.how_smooth;
    let compression_factor = params.compression_factor;
    let node_addition_threshold = params.node_addition_threshold;

    let (outer_changes, inner_changes) = manual_neighbor_changes(ts, node_change, OUTER, INNER, how_smooth, compression_factor, stitch);
    let energy_state = energy(ts, initial_gray_matter_area);
    apply_changes(&mut ts.layers[OUTER], &outer_changes);
    apply_changes(&mut ts.layers[INNER], &inner_changes);
    let energy_neighbor = energy(ts, initial_gray_matter_area);

    intersection_effects(ts, &outer_changes, &inner_changes, energy_state, energy_neighbor, temperature, rng);
    add_single_node_effects(ts, OUTER, node_addition_threshold);
    add_single_node_effects(ts, INNER, node_addition_threshold);

    vec![outer_changes, inner_changes]
}
