use types::{ThickSurface, NodeChange};
use graph_change::{apply_changes, revert_changes};
use graph::thick_surface_to_lines;
use vector_2d_helpers::lines_intersection;

const SOME_HUGE_FUCKIN_VALUE: f64 = 1000_000_000.0;

fn neighbor_changes(ts: &ThickSurface) -> (Vec<NodeChange>, Vec<NodeChange>) {
    (Vec::new(),Vec::new())
}

fn energy(ts: &ThickSurface) -> f64 {
    1.0
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

pub fn step(ts: &mut ThickSurface, temperature: f64) {
    let (outer_changes, inner_changes) = neighbor_changes(ts);

    let energy_state = energy(ts);
    apply_changes(&mut ts.outer, &outer_changes);
    apply_changes(&mut ts.inner, &inner_changes);
    let energy_neighbor = energy(ts);

    let lines = thick_surface_to_lines(ts);
    match lines_intersection(&lines) {
        Some(_) => {
            revert_changes(&mut ts.outer, &outer_changes);
            revert_changes(&mut ts.outer, &inner_changes);
        }
        None => {
            let coin_flip = 0.5;
            if probability(energy_state, energy_neighbor, temperature) < coin_flip {
                revert_changes(&mut ts.outer, &outer_changes);
                revert_changes(&mut ts.outer, &inner_changes);
            }
        }
    }
}
