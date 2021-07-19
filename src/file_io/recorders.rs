use graph;
use simulated_annealing;

use std::fs::File;
use std::io::Write;

use graph::types::{ThickSurface, INNER, OUTER};
use simulated_annealing::SimState;
use types::Params;
use std::collections::HashMap;

type RecorderFn = for<'r, 's> fn(&'r ThickSurface, &'s Params) -> f64;

pub struct RecordingState {
    pub f: File,
    pub last_recorded: Vec<f64>,
}

impl RecordingState {
    pub fn empty_state(file_path: &str) -> Option<RecordingState> {
        return match File::create(file_path) {
            Ok(mut f) => Some(RecordingState{
                f,
                last_recorded: vec![]
            }),
            Err(e) => panic!("Couldn't write to file: {:?}", e),
        };
    }
    pub fn initial_state(p: &Params) -> Option<RecordingState> {
        if !p.recorders.is_empty() {
            let mut header = String::new();
            header.push_str("timestep");
            for r in &p.recorders {
                header.push_str(",");
                header.push_str(r);
            }
            header.push_str("\n");

            return match File::create(&p.output_file_path) {
                Ok(mut f) => match f.write_all(header.as_bytes()) {
                    Ok(_) => Some(RecordingState {
                        f,
                        last_recorded: Vec::new(),
                    }),
                    Err(e) => panic!("Couldn't write to file: {:?}", e),
                },
                Err(_) => None,
            };
        }
        None
    }
}

fn outer_perimeter(ts: &ThickSurface, _p: &Params) -> f64 {
    graph::perimeter(&ts.layers[OUTER])
}

fn inner_perimeter(ts: &ThickSurface, _p: &Params) -> f64 {
    graph::perimeter(&ts.layers[INNER])
}

fn outer_area(ts: &ThickSurface, _p: &Params) -> f64 {
    graph::area(&ts.layers[OUTER])
}

fn inner_area(ts: &ThickSurface, _p: &Params) -> f64 {
    graph::area(&ts.layers[INNER])
}

fn energy(ts: &ThickSurface, p: &Params) -> f64 {
    simulated_annealing::energy(ts, p.initial_gray_matter_area)
}

fn gray_matter_area(ts: &ThickSurface, _p: &Params) -> f64 {
    graph::gray_matter_area(ts)
}

fn num_inner_points(ts: &ThickSurface, _p: &Params) -> f64 {
    ts.layers[INNER].nodes.len() as f64
}
fn num_outer_points(ts: &ThickSurface, _p: &Params) -> f64 {
    ts.layers[OUTER].nodes.len() as f64
}

fn name_to_fn(n: &str) -> Option<RecorderFn> {
    match n {
        "energy" => Some(energy),
        "outer perimeter" => Some(outer_perimeter),
        "inner perimeter" => Some(inner_perimeter),
        "outer area" => Some(outer_area),
        "inner area" => Some(inner_area),
        "gray matter area" => Some(gray_matter_area),
        "num inner points" => Some(num_inner_points),
        "num outer points" => Some(num_outer_points),
        _ => None,
    }
}

pub fn rec_map() -> HashMap<String, RecorderFn> {
    let mut r = HashMap::new();
    r.insert(String::from("energy"), energy as for<'r, 's> fn(&'r graph::types::ThickSurface, &'s Params) -> f64);
    r.insert(String::from("outer perimeter"), outer_perimeter as for<'r, 's> fn(&'r graph::types::ThickSurface, &'s Params) -> f64);
    r.insert(String::from("inner perimeter"), inner_perimeter as for<'r, 's> fn(&'r graph::types::ThickSurface, &'s Params) -> f64);
    r.insert(String::from("outer area"), outer_area as for<'r, 's> fn(&'r graph::types::ThickSurface, &'s Params) -> f64);
    r.insert(String::from("inner area"), inner_area as for<'r, 's> fn(&'r graph::types::ThickSurface, &'s Params) -> f64);
    r.insert(String::from("gray matter area"), gray_matter_area as for<'r, 's> fn(&'r graph::types::ThickSurface, &'s Params) -> f64);
    r.insert(String::from("num inner points"), num_inner_points as for<'r, 's> fn(&'r graph::types::ThickSurface, &'s Params) -> f64);
    r.insert(String::from("num outer points"), num_outer_points as for<'r, 's> fn(&'r graph::types::ThickSurface, &'s Params) -> f64);
    r
}

pub fn record(sim_state: &SimState, p: &Params, recording_state: &mut RecordingState) {
    let mut line = String::new();
    let mut new_vals = Vec::new();
    for r in &p.recorders {
        let val = match name_to_fn(r) {
            Some(recorder) => recorder(&sim_state.ts, p),
            None => panic!(format!("unsupported recorder: {}", r)),
        };
        new_vals.push(val);
        line.push_str(format!(",{}", val).as_str());
    }
    line.push_str("\n");
    if line.len() > 0 {
        line.remove(0);
    }; // remove leading comma
    if true
    /* new_vals != recording_state.last_recorded */
    {
        line.insert_str(0, &*format!("{},", sim_state.timestep));
        match recording_state.f.write_all(line.as_bytes()) {
            Ok(_) => {}
            Err(e) => panic!("Couldn't write to file: {:?}", e),
        }
        recording_state.last_recorded = new_vals;
    }
}
