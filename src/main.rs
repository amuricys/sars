mod graph;
mod graph_change;
mod recorders;
mod renderer;
mod simulated_annealing;
mod stitcher;
mod types;
mod vector_2d_helpers;

extern crate float_cmp;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;
extern crate toml;
extern crate vec1;

use std::f64::consts::PI;
use renderer::draw_mode_rendering;

fn toml_table_to_params(table: toml::Value) -> types::Params {
    match table {
        toml::Value::Table(m) => {
            let initial_radius = m.get("initial_radius").unwrap().as_float().unwrap();
            let initial_thickness = m.get("initial_thickness").unwrap().as_float().unwrap();
            let initial_area = 2.0 * PI * initial_radius - (2.0 * PI * (initial_radius - initial_thickness));
            types::Params {
                initial_thickness: initial_thickness,
                initial_radius: initial_radius,
                initial_gray_matter_area: initial_area,
                initial_num_points: m.get("initial_num_points").unwrap().as_integer().unwrap() as usize,
                initial_temperature: m.get("initial_temperature").unwrap().as_float().unwrap(),
                compression_factor: m.get("compression_factor").unwrap().as_float().unwrap(),
                softness_factor: m.get("softness_factor").unwrap().as_float().unwrap(),
                how_smooth: m.get("how_smooth").unwrap().as_integer().unwrap() as usize,
                node_addition_threshold: m.get("node_addition_threshold").unwrap().as_float().unwrap(),
                node_deletion_threshold: m.get("node_deletion_threshold").unwrap().as_float().unwrap(),
                low_high: (
                    m.get("low_high").unwrap().as_array().unwrap()[0].as_float().unwrap(),
                    m.get("low_high").unwrap().as_array().unwrap()[1].as_float().unwrap(),
                ),
                recorders: m
                    .get("recorders")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| String::from(x.as_str().unwrap()))
                    .collect(),
            }
        }
        _ => panic!("No key-value table found in parameters.toml"),
    }
}

fn real_main() {
    let params: types::Params = match std::fs::read_to_string("parameters.toml") {
        Err(_) => panic!("No parameters.toml file found in directory"),
        Ok(content) => toml_table_to_params(content.parse::<toml::Value>().unwrap()),
    };
    let mut my_graph = graph::circular_thick_surface(params.initial_radius, params.initial_thickness, params.initial_num_points);
    let mut rng = rand::thread_rng();

    let (mut renderer, mut window) = renderer::setup_renderer();

    renderer::setup_optimization_and_loop(
        &mut my_graph,
        &mut rng,
        &mut window,
        &mut renderer,
        |ts, _, s| renderer::lines_from_thick_surface(ts, s),
        &params,
    )
}

fn playin_main() {
    let (mut renderer, mut window) = renderer::setup_renderer();
    draw_mode_rendering(&mut window, &mut renderer)
}

fn main() {
    playin_main()
    // real_main()
}
