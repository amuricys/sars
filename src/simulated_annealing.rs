use graph;
use graph::circular_thick_surface;
use graph::effects::{add_node_, apply_changes, changer_of_choice, merge_nodes_, random_change, revert_changes, smooth_change_out};
use graph::types::{NodeChange, NodeChangeMap, Smooth, ThickSurface, INNER, OUTER};
use linalg_helpers::lines_intersection;
use rand::prelude::ThreadRng;
use rand::Rng;
use stitcher::stitch_default;
use stitcher::types::Stitching;
use types::Params;

const PRACTICALLY_INFINITY: f64 = 100_000_000.0;

fn neighbor_changes(
    ts: &ThickSurface,
    layer_to_push: usize,
    layer_across: usize,
    how_smooth: usize,
    compression_factor: f64,
    stitch: &Stitching,
    low_high: (f64, f64),
    rng: &mut rand::rngs::ThreadRng,
) -> (NodeChangeMap, NodeChangeMap) {
    let outer_change = random_change(&ts.layers[layer_to_push], low_high, rng);
    let smoothed_changes = smooth_change_out(&ts.layers[layer_to_push], outer_change.clone(), Smooth::Count(how_smooth));
    let smoothed_inner_changes = changer_of_choice(
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

fn intersection_effects(
    ts: &mut ThickSurface,
    outer_changes: &NodeChangeMap,
    inner_changes: &NodeChangeMap,
    energy_state: f64,
    energy_neighbor: f64,
    temperature: f64,
    rng: &mut rand::rngs::ThreadRng,
) {
    let lines1 = graph::graphs_to_lines(&ts.layers);
    let coin_flip = rng.gen_range(0.0, 1.0);
    match lines_intersection(&lines1) {
        Some(_) => {
            revert_changes(&mut ts.layers[OUTER], outer_changes);
            revert_changes(&mut ts.layers[INNER], inner_changes);

            let lines2 = graph::graphs_to_lines(&ts.layers);
            match lines_intersection(&lines2) {
                Some(_) => {
                    println!("Howwwwwwww?? lines1:\n{:?}\nlines2:\n{:?}", lines1, lines2);
                    panic!("caceta")
                }
                None => {}
            }
        }
        None => {
            if probability_to_accept_neighbor_state(energy_state, energy_neighbor, temperature) < coin_flip {
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
                add_node_(ts, layer_to_add, &addition);
                // println!("addition: {:?}", addition);
                // println!("prev: {:?}\nnext: {:?}\n", ts.layers[layer_to_add].nodes[addition.n.prev_id], ts.layers[layer_to_add].nodes[addition.n.next_id]);
                break; // THE BREAK IS WHAT LETS THIS WORK, GODDAMN
            }
            None => {}
        }
    }
}

fn aaa(ts: &ThickSurface) {
    let mut n = &ts.layers[OUTER].nodes[0];
    // println!("AAA");
    loop {
        n = n.next_by(&ts.layers[OUTER], 1);
        if n == &ts.layers[OUTER].nodes[0] {
            break;
        }
    }
    // println!("BBB");
}

fn delete_single_node_effects(ts: &mut ThickSurface, layer_from_which_delete: usize, deletion_threshold: f64, max_merge_steps_away: usize) {
    let graph_from_which_delete = &ts.layers[layer_from_which_delete];
    for n in &graph_from_which_delete.nodes {
        match graph::nodes_to_merge(ts, layer_from_which_delete, n, deletion_threshold, max_merge_steps_away, true) {
            Some(deletion) => {
                merge_nodes_(ts, &deletion);

                let lines = graph::graphs_to_lines(&ts.layers);
                match lines_intersection(&lines) {
                    Some(_) => println!("How??"),
                    None => {}
                }

                break; // THE BREAK IS WHAT LETS THIS WORK, GODDAMN
            }
            None => {}
        }
    }
    aaa(ts)
}

#[derive(Clone, Debug)]
pub struct SimState {
    pub ts: ThickSurface,
    pub temperature: f64,
    pub stitching: Stitching,
    pub timestep: u64,
    pub rng: ThreadRng,
}

impl SimState {
    pub fn initial_state(p: &Params) -> SimState {
        let ts = circular_thick_surface(p.initial_radius, p.initial_thickness, p.initial_num_points);
        let s = stitch_default(&ts);
        let mut rng = rand::thread_rng();

        SimState {
            ts: ts,
            temperature: p.initial_temperature,
            stitching: s,
            timestep: 0,
            rng: rng,
        }
    }
}

pub fn step(sim_state: &mut SimState, params: &Params) -> Vec<NodeChangeMap> {
    let how_smooth = params.how_smooth;
    let compression_factor = params.compression_factor;
    let low_high = params.low_high;
    let node_addition_threshold = params.node_addition_threshold;
    let node_deletion_threshold = params.node_deletion_threshold;

    let (outer_changes, inner_changes) = neighbor_changes(
        &sim_state.ts,
        OUTER,
        INNER,
        how_smooth,
        compression_factor,
        &sim_state.stitching,
        low_high,
        &mut sim_state.rng,
    );

    let energy_state = energy(&sim_state.ts, params.initial_gray_matter_area);
    apply_changes(&mut sim_state.ts.layers[OUTER], &outer_changes);
    apply_changes(&mut sim_state.ts.layers[INNER], &inner_changes);
    let energy_neighbor = energy(&sim_state.ts, params.initial_gray_matter_area);

    intersection_effects(
        &mut sim_state.ts,
        &outer_changes,
        &inner_changes,
        energy_state,
        energy_neighbor,
        sim_state.temperature,
        &mut sim_state.rng,
    );
    add_single_node_effects(&mut sim_state.ts, OUTER, node_addition_threshold);
    add_single_node_effects(&mut sim_state.ts, INNER, node_addition_threshold);

    delete_single_node_effects(&mut sim_state.ts, OUTER, node_deletion_threshold, params.max_merge_steps_away);
    delete_single_node_effects(&mut sim_state.ts, INNER, node_deletion_threshold, params.max_merge_steps_away);

    sim_state.temperature = temperature(sim_state, params.temperature_param);
    sim_state.timestep += 1;
    vec![outer_changes, inner_changes]
}
