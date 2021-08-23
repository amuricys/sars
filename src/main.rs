#![recursion_limit = "256"]

mod file_io;
mod graph;
mod linalg_helpers;
mod my_gui;
mod renderer;
mod shared_shit;
mod simulated_annealing;
mod simulated_annealing_dumber_and_better;
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
extern crate geo;
extern crate num_traits;
extern crate piston_window;
extern crate regex;

use renderer::draw_mode::draw_mode_rendering;
use std::env;
use simulated_annealing::energy;
use graph::area;
use graph::types::OUTER;
use file_io::recorders;
use toml::from_str;

fn real_main() {
    let params: types::Params = match std::fs::read_to_string("parameters.toml") {
        Err(_) => panic!("No parameters.toml file found in directory"),
        Ok(content) => file_io::toml_table_to_params(content.parse::<toml::Value>().unwrap()),
    };
    let (mut renderer, mut window) = renderer::setup_renderer();
    let mut sim_state = simulated_annealing::SimState::initial_state(&params);

    renderer::setup_optimization_and_loop(
        &mut sim_state,
        &mut window,
        &mut renderer,
        |ss| renderer::lines_from_thick_surface(&ss.ts),
        &params,
    )
}

fn no_gui_main(params_file_path: &str, how_many_reps: u64) {
    let params: types::Params = match std::fs::read_to_string(params_file_path) {
        Err(_) => panic!(format!("Parameter file named \"{}\" not found.", params_file_path)),
        Ok(content) => file_io::toml_table_to_params(content.parse::<toml::Value>().unwrap()),
    };
    
   
    let mut recording_state = recorders::RecordingState::initial_state(&params).unwrap_or_else(|| {panic!("Couldn't create recording state")});
    let mut sim_state = simulated_annealing::SimState::initial_state(&params);
        
    loop {
        simulated_annealing_dumber_and_better::step(&mut sim_state, &params);
        recorders::record(&sim_state, &params, &mut recording_state);
        
        
        if sim_state.timestep > how_many_reps { // Não sei de onde tirar esse número
                   
            break;            
        }
    }

}

fn no_gui_main_rep(params_file_path: &str
    , how_many_reps: u64
) {
    
    let params: types::Params = match std::fs::read_to_string(params_file_path) {
        Err(_) => panic!(format!("Parameter file named \"{}\" not found.", params_file_path)),
        Ok(content) => file_io::toml_table_to_params(content.parse::<toml::Value>().unwrap()),
    };
    
   
    let mut recording_state = recorders::RecordingState::initial_state(&params).unwrap_or_else(|| {panic!("Couldn't create recording state")});
    for _i in 0..50{
        let mut sim_state = simulated_annealing::SimState::initial_state(&params);
    
    loop {
        simulated_annealing_dumber_and_better::step(&mut sim_state, &params);
        
        if sim_state.timestep > how_many_reps { // Não sei de onde tirar esse número
            recorders::record(&sim_state, &params, &mut recording_state);        
            break;            
        }
    }
}
}

fn playin_main() {
    let (mut renderer, mut window) = renderer::setup_renderer();
    draw_mode_rendering(&mut window, &mut renderer)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let empty_vec: Vec<i64> = Vec::new();
    let s: Vec<i64> = empty_vec.iter().map(|x| *x).collect();
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
    } else if args[1] == "no_gui" {
        no_gui_main(&args[2]
        , args[3].parse::<u64>().unwrap())
        ;
    }else if args[1] == "no_gui_rep" {
    no_gui_main_rep(&args[2]
    , args[3].parse::<u64>().unwrap())
    ;
    }
}
