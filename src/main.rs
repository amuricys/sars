#![recursion_limit = "256"]

mod file_io;
mod graph;
mod linalg_helpers;
mod my_gui;
mod renderer;
mod shared_shit;
mod simulated_annealing;
mod stitcher;
mod types;

extern crate float_cmp;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate pathfinding;
extern crate piston;
extern crate rand;
extern crate toml;
extern crate vec1;

extern crate conrod_core;
extern crate conrod_piston;
extern crate find_folder;
extern crate num_traits;
extern crate piston_window;
extern crate regex;

use renderer::draw_mode::draw_mode_rendering;
use std::env;
use stitcher::stitch_choice;

fn real_main() {
    let params: types::Params = match std::fs::read_to_string("parameters.toml") {
        Err(_) => panic!("No parameters.toml file found in directory"),
        Ok(content) => file_io::toml_table_to_params(content.parse::<toml::Value>().unwrap()),
    };
    let mut my_graph = graph::circular_thick_surface(params.initial_radius, params.initial_thickness, params.initial_num_points);
    let mut rng = rand::thread_rng();

    let (mut renderer, mut window) = renderer::setup_renderer();

    let mut sim_state = simulated_annealing::SimState::initial_state(&params);

    renderer::setup_optimization_and_loop(
        &mut sim_state,
        &mut rng,
        &mut window,
        &mut renderer,
        |ss| renderer::lines_from_thick_surface(&ss.ts),
        &params,
    )
}

fn playin_main() {
    let (mut renderer, mut window) = renderer::setup_renderer();
    draw_mode_rendering(&mut window, &mut renderer)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        real_main()
    } else if args[1] == "debug" {
        playin_main()
    } else if args[1] == "smart" {
        println!("Path hehe: djumba");
    } else if args[1] == "conrod" {
        shared_shit::conrod_main();
    } else if args[1] == "my_gui" {
        my_gui::my_ui_main();
    }
}
