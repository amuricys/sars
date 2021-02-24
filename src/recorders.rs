use graph;
use simulated_annealing;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::iter::FromIterator;
use types::{Params, ThickSurface, INNER, OUTER};

type RecorderFn = for<'r, 's> fn(&'r ThickSurface, &'s Params) -> f64;

pub fn outer_perimeter(ts: &ThickSurface, _p: &Params) -> f64 {
    graph::perimeter(&ts.layers[OUTER])
}

pub fn inner_perimeter(ts: &ThickSurface, _p: &Params) -> f64 {
    graph::perimeter(&ts.layers[INNER])
}

pub fn outer_area(ts: &ThickSurface, _p: &Params) -> f64 {
    graph::area(&ts.layers[OUTER])
}

pub fn inner_area(ts: &ThickSurface, _p: &Params) -> f64 {
    graph::area(&ts.layers[INNER])
}

pub fn energy(ts: &ThickSurface, p: &Params) -> f64 {
    simulated_annealing::energy(ts, p.initial_gray_matter_area)
}

fn name_to_fn(n: &str) -> Option<RecorderFn> {
    match n {
        "energy" => Some(energy),
        "outer perimeter" => Some(outer_perimeter),
        "inner perimeter" => Some(inner_perimeter),
        "outer area" => Some(outer_area),
        "inner area" => Some(inner_area),
        _ => None,
    }
}

pub fn create_file_with_header(file_path: &str, recorders: &Vec<String>) -> Option<File> {
    if !recorders.is_empty() {
        let mut header = String::new();
        for r in recorders {
            header.push_str(",");
            header.push_str(r);
        }
        header.push_str("\n");
        if header.len() > 0 {
            header.remove(0);
        }; // remove leading comma

        return match File::create(file_path) {
            Ok(mut f) => {
                f.write_all(header.as_bytes());
                Some(f)
            }
            Err(_) => None,
        };
    }
    None
}

pub fn record(ts: &ThickSurface, p: &Params, f: &mut File) {
    let mut line = String::new();
    for r in &p.recorders {
        let val = match name_to_fn(r) {
            Some(recorder) => recorder(ts, p),
            None => panic!(format!("unsupported recorder: {}", r)),
        };
        line.push_str(format!(",{}", val).as_str());
    }
    line.push_str("\n");
    if line.len() > 0 {
        line.remove(0);
    }; // remove leading comma
    f.write_all(line.as_bytes());
}
