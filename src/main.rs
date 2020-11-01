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

struct Params {
    initial_thickness: f64,
    initial_radius: f64,
    initial_num_points: usize,
    initial_temperature: f64,
    compression_factor: f64,
    how_smooth: usize
}

fn toml_table_to_params(table: toml::Value) -> Params {
    match table {
        toml::Value::Table(m) => Params {
                initial_thickness: m.get("initial_thickness").unwrap().as_float().unwrap(),
                initial_radius: m.get("initial_radius").unwrap().as_float().unwrap(),
                initial_num_points: m.get("initial_num_points").unwrap().as_integer().unwrap() as usize,
                initial_temperature: m.get("initial_temperature").unwrap().as_float().unwrap(),
                compression_factor: m.get("compression_factor").unwrap().as_float().unwrap(),
                how_smooth: m.get("how_smooth").unwrap().as_integer().unwrap() as usize,
            },
        _ => panic!("No key-value table found in parameters.toml")
    }
}

fn main() {
    let params: Params = match std::fs::read_to_string("parameters.toml") {
        Err(_) => panic!("No parameters.toml file found in directory"),
        Ok(content) => toml_table_to_params(content.parse::<toml::Value>().unwrap())
    };
    let mut my_graph = graph::thick_surface(params.initial_radius, params.initial_thickness,  params.initial_num_points);
    let mut rng = rand::thread_rng();

    let (mut renderer, mut window) = renderer::setup_renderer();
    
    renderer::setup_optimization_and_loop(&mut my_graph, &mut rng, &mut window, &mut renderer, params.initial_temperature, params.compression_factor, params.how_smooth)
}
