mod consts;
pub mod draw_mode;
mod junk;
mod types;

use glutin_window::GlutinWindow as Window;
use piston::event_loop::{EventSettings, Events};
use piston::input::{MouseCursorEvent, RenderEvent, UpdateEvent};
use piston::window::WindowSettings;

use graph;

use file_io::recorders;
use piston::{Button, Event, PressEvent};
use simulated_annealing;
use simulated_annealing_dumber_and_better;

use graph::types::{NodeChange, NodeChangeMap, Smooth, ThickSurface, INNER, OUTER};
use renderer::types::Line;
use simulated_annealing::SimState;
use stitcher::types::{Stitching, Strategy};
use stitcher::{stitch_choice, stitch_default};
use types::Params;

pub fn lines_from_thick_surface(ts: &ThickSurface) -> Vec<types::Line> {
    let mut lines = Vec::new();
    let color_array = [consts::PINK, consts::BLUE, consts::PURPLE];
    for i in 0..ts.layers.len() {
        let g = &ts.layers[i];
        for node in &g.nodes {
            lines.push(types::Line {
                points: (node.x, node.y, node.next(g).x, node.next(g).y),
                color: color_array[i],
            });
        }
    }
    // for (k, v) in &v[OUTER] {
    //     let outer_x = ts.layers[OUTER].nodes[*k].x;
    //     let outer_y = ts.layers[OUTER].nodes[*k].y;
    //     for val in v {
    //         let inner_x = ts.layers[INNER].nodes[val.0].x;
    //         let inner_y = ts.layers[INNER].nodes[val.0].y;
    //         lines.push(types::Line {
    //             points: (outer_x, outer_y, inner_x, inner_y),
    //             color: consts::PURPLE,
    //         });
    //     }
    // }
    lines
}

#[derive(Debug, PartialOrd, PartialEq)]
pub enum StepType {
    ManualChange,
    OneAtATime,
    Automatic,
    NoStep,
    Reset,
}

#[derive(Debug, PartialOrd, PartialEq)]
struct RenderState {
    pub should_step: bool,
    pub one_at_a_time: bool,
    pub step_type: StepType,
    pub hyper_debug: bool,
}

fn next_state(event: Option<Button>, s: RenderState) -> RenderState {
    match event {
        Some(piston::Button::Keyboard(piston::Key::Space)) => RenderState {
            should_step: !s.should_step,
            one_at_a_time: !s.one_at_a_time,
            step_type: match s.step_type {
                StepType::Automatic => StepType::NoStep,
                _ => StepType::Automatic,
            },
            ..s
        },
        Some(piston::Button::Keyboard(piston::Key::N)) => RenderState {
            step_type: if s.one_at_a_time { StepType::OneAtATime } else { s.step_type },
            ..s
        },
        Some(piston::Button::Keyboard(piston::Key::M)) => RenderState {
            step_type: if s.one_at_a_time { StepType::ManualChange } else { s.step_type },
            ..s
        },
        Some(piston::Button::Keyboard(piston::Key::R)) => RenderState {
            should_step: false,
            one_at_a_time: true,
            step_type: StepType::Reset,
            ..s
        },
        Some(piston::Button::Keyboard(piston::Key::H)) => RenderState {
            hyper_debug: !s.hyper_debug,
            ..s
        },
        _ => RenderState {
            step_type: if !s.should_step { StepType::NoStep } else { s.step_type },
            ..s
        },
    }
}

fn initial_render_state() -> RenderState {
    RenderState {
        should_step: false,
        one_at_a_time: true,
        step_type: StepType::NoStep,
        hyper_debug: false,
    }
}

fn lines_from_change_map(ts: &ThickSurface, change_maps: Vec<NodeChangeMap>) -> Vec<types::Line> {
    let mut ret = Vec::new();
    for i in 0..ts.layers.len() {
        for (_, c) in &change_maps[i] {
            let (cs_next_x, cs_next_y) = match change_maps[i].get(&ts.layers[i].nodes[c.id].next_id) {
                Some(cs_next_which_was_also_changed) => (
                    cs_next_which_was_also_changed.cur_x + cs_next_which_was_also_changed.delta_x,
                    cs_next_which_was_also_changed.cur_y + cs_next_which_was_also_changed.delta_y,
                ),
                None => (
                    ts.layers[i].nodes[ts.layers[i].nodes[c.id].next_id].x,
                    ts.layers[i].nodes[ts.layers[i].nodes[c.id].next_id].y,
                ),
            };
            let (cs_prev_x, cs_prev_y) = match change_maps[i].get(&ts.layers[i].nodes[c.id].prev_id) {
                Some(cs_prev_which_was_also_changed) => (
                    cs_prev_which_was_also_changed.cur_x + cs_prev_which_was_also_changed.delta_x,
                    cs_prev_which_was_also_changed.cur_y + cs_prev_which_was_also_changed.delta_y,
                ),
                None => (
                    ts.layers[i].nodes[ts.layers[i].nodes[c.id].prev_id].x,
                    ts.layers[i].nodes[ts.layers[i].nodes[c.id].prev_id].y,
                ),
            };
            ret.push(types::Line {
                points: (c.cur_x + c.delta_x, c.cur_y + c.delta_y, cs_next_x, cs_next_y),
                color: consts::BLUE,
            });
            ret.push(types::Line {
                points: (c.cur_x + c.delta_x, c.cur_y + c.delta_y, cs_prev_x, cs_prev_y),
                color: consts::BLUE,
            });
            // let (reference_x, reference_y) = bisecting_vector(c.cur_x + c.delta_x, c.cur_y + c.delta_y, cs_next_x, cs_next_y, cs_prev_x, cs_prev_y);
            // ret.push(types::Line {points: (c.cur_x + c.delta_x, c.cur_y + c.delta_y, reference_x, reference_y), color: consts::GREEN});
        }
    }
    ret
}

fn maybe_imaginary_lines(state: &RenderState, e: &Event, sim_state: &SimState, params: &Params, imaginary_lines: Vec<Line>) -> Vec<Line> {
    if !state.hyper_debug {
        Vec::new()
    } else {
        match e.mouse_cursor_args() {
            Some([x, y]) => {
                let (cursor_pos_x, cursor_pos_y) = junk::from_window_to_minus1_1(x, y, consts::WINDOW_SIZE.0, consts::WINDOW_SIZE.1);
                let closest_node = graph::closest_node_to_some_point(&sim_state.ts.layers[OUTER], cursor_pos_x, cursor_pos_y);
                let imaginary_change = NodeChange {
                    id: closest_node.id,
                    cur_x: closest_node.x,
                    cur_y: closest_node.y,
                    delta_x: cursor_pos_x - closest_node.x,
                    delta_y: cursor_pos_y - closest_node.y,
                };
                let surrounding_imaginary_changes =
                    graph::effects::smooth_change_out(&sim_state.ts.layers[OUTER], imaginary_change, Smooth::Count(params.how_smooth));
                let inner_imaginary_changes = graph::effects::changer_of_choice(
                    &sim_state.ts.layers[INNER],
                    &sim_state.ts.layers[OUTER],
                    &surrounding_imaginary_changes,
                    0.0,
                    &sim_state.stitching,
                );
                lines_from_change_map(&sim_state.ts, vec![surrounding_imaginary_changes, inner_imaginary_changes])
            }
            None => imaginary_lines,
        }
    }
}

pub fn setup_optimization_and_loop<F>(
    sim_state: &mut SimState,
    window: &mut Window,
    renderer: &mut types::Renderer,
    how_to_make_lines: F,
    params: &Params,
) where
    F: Fn(&SimState) -> Vec<types::Line>,
{
    let mut render_state = initial_render_state();
    let mut recording_state = recorders::RecordingState::initial_state(&params);
    let mut events = Events::new(EventSettings::new());
    let mut imaginary_lines = Vec::new();

    while let Some(e) = events.next(window) {
        imaginary_lines = maybe_imaginary_lines(&render_state, &e, &sim_state, params, imaginary_lines);
        let mut lines = how_to_make_lines(&sim_state);
        lines.append(&mut imaginary_lines.clone()); // I really don't get why there isn't a good immutable append operation

        if let Some(args) = e.render_args() {
            renderer.render(&args, &lines);
        }

        if let Some(args) = e.update_args() {
            renderer.update(&args);
        }

        render_state = next_state(e.press_args(), render_state);
        match render_state.step_type {
            StepType::Automatic => simulated_annealing_dumber_and_better::step(sim_state, params), // simulated_annealing::step(sim_state, params),
            StepType::Reset => *sim_state = simulated_annealing::SimState::initial_state(params),
            _ => {}
        }
        match &mut recording_state {
            Some(f) => recorders::record(&sim_state, params, f),
            None => {}
        }
        // Se vc quer que pare de rodar
        if sim_state.timestep >= 10000 {
            break
        }
        // Se vc quer que recomece
        let timestamp = sim_state.timestep;
        if timestamp >= 10000 {
            *sim_state = SimState::initial_state(params);
        }
        // Se vc quer rodar de um programa EXTERNO, tbm é uma opção
    }
}

pub fn setup_renderer() -> (types::Renderer, Window) {
    // Create an Glutin window.
    let window: Window = WindowSettings::new("spinning-square", consts::WINDOW_SIZE)
        .graphics_api(types::Renderer::gl_ver())
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let app = types::Renderer::new();

    (app, window)
}
