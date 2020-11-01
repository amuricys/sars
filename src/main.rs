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
    initial_num_points: i64,
    initial_temperature: f64,
    compression_factor: f64,
}

fn main() {
    /* Parameter setting TODO: Move to some config file that is read on startup */
    let initial_thickness = 0.02;
    let initial_radius = 1.0;
    let initial_num_points = 200;
    let initial_temperature = 0.0;
    let compression_factor = 1.2;
    let mut my_graph = graph::thick_surface(initial_radius, initial_thickness,  initial_num_points);
    let mut rng = rand::thread_rng();

    let (mut renderer, mut window) = renderer::setup_renderer();
    
    renderer::setup_optimization_and_loop(&mut my_graph, &mut rng, &mut window, &mut renderer, initial_temperature, compression_factor, 10)
}
