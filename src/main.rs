mod simulated_annealing;
mod vector_2d_helpers;
mod renderer;
mod graph_change;
mod types;
mod graph;

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;
extern crate float_cmp;
extern crate toml;
extern crate vec1;

struct Params {
    initial_thickness: f64,
    initial_radius: f64,
    initial_num_points: usize,
    initial_temperature: f64,
    compression_factor: f64,
    softness_factor: f64,    // <- how much should closeness of nodes in different surfaces impact pushes?
    how_smooth: usize,
    node_addition_threshold: f64,
    node_deletion_threshold: f64,
    low_high: (f64, f64)
}

fn toml_table_to_params(table: toml::Value) -> Params {
    match table {
        toml::Value::Table(m) => Params {
            initial_thickness: m.get("initial_thickness").unwrap().as_float().unwrap(),
            initial_radius: m.get("initial_radius").unwrap().as_float().unwrap(),
            initial_num_points: m.get("initial_num_points").unwrap().as_integer().unwrap() as usize,
            initial_temperature: m.get("initial_temperature").unwrap().as_float().unwrap(),
            compression_factor: m.get("compression_factor").unwrap().as_float().unwrap(),
            softness_factor: m.get("softness_factor").unwrap().as_float().unwrap(),
            how_smooth: m.get("how_smooth").unwrap().as_integer().unwrap() as usize,
            node_addition_threshold: m.get("node_addition_threshold").unwrap().as_float().unwrap(),
            node_deletion_threshold: m.get("node_deletion_threshold").unwrap().as_float().unwrap(),
            low_high: (m.get("low_high").unwrap().as_array().unwrap()[0].as_float().unwrap(), m.get("low_high").unwrap().as_array().unwrap()[1].as_float().unwrap())
        },
        _ => panic!("No key-value table found in parameters.toml")
    }
}

fn real_main() {
    let params: Params = match std::fs::read_to_string("parameters.toml") {
        Err(_) => panic!("No parameters.toml file found in directory"),
        Ok(content) => toml_table_to_params(content.parse::<toml::Value>().unwrap())
    };
    let mut my_graph = graph::circular_thick_surface(params.initial_radius, params.initial_thickness, params.initial_num_points);
    let mut rng = rand::thread_rng();

    let (mut renderer, mut window) = renderer::setup_renderer();

    renderer::setup_optimization_and_loop(&mut my_graph,
                                          &mut rng,
                                          &mut window,
                                          &mut renderer,
                                          params.initial_temperature,
                                          params.compression_factor,
                                          params.how_smooth,
                                          params.node_addition_threshold,
                                          params.node_deletion_threshold,
                                          params.low_high)
}

fn playground_main() {
    let mut my_graph = graph::debug_straight_surface(30);
    let (mut renderer,
        mut window) = renderer::setup_renderer();
    renderer::render_playground(&mut my_graph, &mut window, &mut renderer, 0, 1.0, 3);
}

fn main() {
    //playground_main()
    real_main()
}
