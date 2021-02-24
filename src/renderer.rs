use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{MouseCursorEvent, RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use graph;
use graph_change;
use piston::{Button, PressEvent};
use recorders;
use simulated_annealing;


use stitcher;
use stitcher::Stitching;
use types::{NodeChange, NodeChangeMap, Params, ThickSurface, INNER, OUTER};


type Color = [f32; 4];

const BLACK: Color = [0.0, 0.0, 0.0, 0.0];
const WHITE: Color = [1.0, 1.0, 1.0, 1.0];
const PURPLE: Color = [0.8, 0.0, 0.8, 1.0];
const PINK: Color = [1.0, 0.4, 1.0, 1.0];
const BLUE: Color = [0.2, 0.2, 1.0, 1.0];
const GREEN: Color = [0.2, 1.0, 0.2, 1.0];

pub struct Renderer {
    gl: GlGraphics,
    // OpenGL drawing backend.
    rotation: f64, // Rotation for the square.
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct Line {
    points: (f64, f64, f64, f64),
    color: Color,
}

/* These two are NOT INVERSES */
fn from_minus1_1_to_window(x: f64, y: f64, window_size_x: f64, window_size_y: f64) -> (f64, f64) {
    (x * window_size_x / 2.0, y * (-window_size_y / 2.0))
}

fn from_window_to_minus1_1(x: f64, y: f64, window_size_x: f64, window_size_y: f64) -> (f64, f64) {
    (2.0 * x / window_size_x - 1.0, (-2.0 * y / window_size_y) + 1.0)
}

impl Renderer {
    fn render(&mut self, args: &RenderArgs, lines: &Vec<Line>) {
        use graphics::*;

        let rotation = self.rotation;
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            let transform = c.transform.trans(x, y).rot_rad(rotation).trans(-0.0, -0.0);

            for l in lines {
                let (x1, y1, x2, y2) = l.points;
                let col = l.color;
                let from = from_minus1_1_to_window(x1, y1, args.window_size[0], args.window_size[1]);
                let to = from_minus1_1_to_window(x2, y2, args.window_size[0], args.window_size[1]);
                line_from_to(col, 0.5, [from.0, from.1], [to.0, to.1], transform, gl);
            }
        });
    }

    fn update(&mut self, _args: &UpdateArgs) {
        // Rotate 2 radians per second.
        // self.rotation += 2.0 * args.dt;
    }
}

pub fn lines_from_thick_surface(ts: &ThickSurface, Stitching::Stitch(v): &Stitching) -> Vec<Line> {
    let mut lines = Vec::new();
    for i in 0..ts.layers.len() {
        let g = &ts.layers[i];
        for node in &g.nodes {
            lines.push(Line {
                points: (node.x, node.y, node.next(g).x, node.next(g).y),
                color: PINK,
            });
        }
    }
    for (k, v) in &v[OUTER] {
        let outer_x = ts.layers[OUTER].nodes[*k].x;
        let outer_y = ts.layers[OUTER].nodes[*k].y;
        for val in v {
            let inner_x = ts.layers[INNER].nodes[val.0].x;
            let inner_y = ts.layers[INNER].nodes[val.0].y;
            lines.push(Line {
                points: (outer_x, outer_y, inner_x, inner_y),
                color: PURPLE,
            });
        }
    }
    lines
}

pub fn lines_playground(ts: &ThickSurface, last_changes: &Vec<NodeChangeMap>) -> Vec<Line> {
    /* Ignores first node and renders node changes for every layer of node changes */
    let mut lines = Vec::new();
    for i in 0..ts.layers.len() {
        let g = &ts.layers[i];
        for node in &g.nodes {
            if node.id != 0 && node.next(g).id != 0 {
                lines.push(Line {
                    points: (node.x, node.y, node.next(g).x, node.next(g).y),
                    color: PINK,
                });
                if i == OUTER {
                    if i == OUTER { /* TODO: get lines based on acrossness map/matrix */ }
                }
            }
        }
    }
    for l in last_changes {
        for (_n, change) in l {
            lines.push(Line {
                points: (
                    change.cur_x,
                    change.cur_y,
                    change.cur_x + change.delta_x,
                    change.cur_y + change.delta_y,
                ),
                color: BLUE,
            })
        }
    }
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
struct State {
    pub should_step: bool,
    pub one_at_a_time: bool,
    pub step_type: StepType,
    pub temperature: f64,
    pub should_stich: bool,
    pub hyper_debug: bool,
}

fn next_state(event: Option<Button>, s: State) -> State {
    match event {
        Some(piston::Button::Keyboard(piston::Key::Space)) => State {
            should_step: !s.should_step,
            one_at_a_time: !s.one_at_a_time,
            step_type: match s.step_type {
                StepType::Automatic => StepType::NoStep,
                _ => StepType::Automatic,
            },
            ..s
        },
        Some(piston::Button::Keyboard(piston::Key::N)) => State {
            step_type: if s.one_at_a_time {
                StepType::OneAtATime
            } else {
                s.step_type
            },
            ..s
        },
        Some(piston::Button::Keyboard(piston::Key::M)) => State {
            step_type: if s.one_at_a_time {
                StepType::ManualChange
            } else {
                s.step_type
            },
            ..s
        },
        Some(piston::Button::Keyboard(piston::Key::R)) => State {
            should_step: false,
            one_at_a_time: true,
            step_type: StepType::Reset,
            temperature: 0.0,
            ..s
        },
        Some(piston::Button::Keyboard(piston::Key::H)) => State {
            hyper_debug: !s.hyper_debug,
            ..s
        },
        _ => State {
            step_type: if !s.should_step { StepType::NoStep } else { s.step_type },
            ..s
        },
    }
}

fn initial_state(initial_temperature: f64) -> State {
    State {
        should_step: false,
        one_at_a_time: true,
        step_type: StepType::NoStep,
        temperature: initial_temperature,
        should_stich: true,
        hyper_debug: false,
    }
}

fn lines_from_change_map(ts: &ThickSurface, change_maps: Vec<NodeChangeMap>) -> Vec<Line> {
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
            ret.push(Line {
                points: (c.cur_x + c.delta_x, c.cur_y + c.delta_y, cs_next_x, cs_next_y),
                color: BLUE,
            });
            ret.push(Line {
                points: (c.cur_x + c.delta_x, c.cur_y + c.delta_y, cs_prev_x, cs_prev_y),
                color: BLUE,
            });
            // let (reference_x, reference_y) = bisecting_vector(c.cur_x + c.delta_x, c.cur_y + c.delta_y, cs_next_x, cs_next_y, cs_prev_x, cs_prev_y);
            // ret.push(Line {points: (c.cur_x + c.delta_x, c.cur_y + c.delta_y, reference_x, reference_y), color: GREEN});
        }
    }
    ret
}

pub fn setup_optimization_and_loop<F>(
    ts: &mut ThickSurface,
    rng: &mut rand::rngs::ThreadRng,
    window: &mut Window,
    renderer: &mut Renderer,
    how_to_make_lines: F,
    params: &Params,
) where
    F: Fn(&ThickSurface, &Vec<NodeChangeMap>, &Stitching) -> Vec<Line>,
{
    let mut state = initial_state(params.initial_temperature);
    let mut stitching = stitcher::stitch(ts);
    let mut events = Events::new(EventSettings::new());
    let mut output_file = recorders::create_file_with_header("output.txt", &params.recorders);
    let mut changeset = vec![];
    let mut imaginary_lines = Vec::new();

    while let Some(e) = events.next(window) {
        let proto_change = NodeChange {
            id: 0,
            cur_x: ts.layers[OUTER].nodes[0].x,
            cur_y: ts.layers[OUTER].nodes[0].y,
            delta_x: -0.2,
            delta_y: 0.0,
        };

        imaginary_lines = if !state.hyper_debug {
            Vec::new()
        } else {
            match e.mouse_cursor_args() {
                Some([x, y]) => {
                    let (cursor_pos_x, cursor_pos_y) = from_window_to_minus1_1(x, y, 800.0, 800.0);
                    let closest_node = graph::closest_node_to_some_point(&ts.layers[OUTER], cursor_pos_x, cursor_pos_y);
                    let imaginary_change = NodeChange {
                        id: closest_node.id,
                        cur_x: closest_node.x,
                        cur_y: closest_node.y,
                        delta_x: cursor_pos_x - closest_node.x,
                        delta_y: cursor_pos_y - closest_node.y,
                    };
                    let surrounding_imaginary_changes =
                        graph_change::smooth_change_out2(&ts.layers[OUTER], imaginary_change, params.how_smooth);
                    let inner_imaginary_changes = graph_change::changes_from_other_graph3(
                        &ts.layers[INNER],
                        &ts.layers[OUTER],
                        &surrounding_imaginary_changes,
                        0.0,
                        stitching.clone(),
                    );
                    lines_from_change_map(ts, vec![surrounding_imaginary_changes, inner_imaginary_changes])
                }
                None => imaginary_lines,
            }
        };
        let mut lines = how_to_make_lines(ts, &changeset, &stitching);
        lines.append(&mut imaginary_lines.clone()); // I really don't get why there isn't a good immutable append operation

        if let Some(args) = e.render_args() {
            renderer.render(&args, &lines);
        }

        if let Some(args) = e.update_args() {
            renderer.update(&args);
        }

        state = next_state(e.press_args(), state);
        match state.step_type {
            StepType::ManualChange => {
                changeset = simulated_annealing::step_with_manual_change(
                    ts,
                    proto_change,
                    params.initial_gray_matter_area,
                    state.temperature,
                    stitching.clone(),
                    params,
                    rng,
                )
            }
            StepType::OneAtATime => {
                changeset = simulated_annealing::step(
                    ts,
                    params.initial_gray_matter_area,
                    state.temperature,
                    stitching.clone(),
                    params,
                    rng,
                )
            }
            StepType::Automatic => {
                changeset = simulated_annealing::step(
                    ts,
                    params.initial_gray_matter_area,
                    state.temperature,
                    stitching.clone(),
                    params,
                    rng,
                )
            }
            StepType::Reset => {
                *ts = {
                    changeset = vec![];
                    graph::circular_thick_surface(
                        params.initial_radius,
                        params.initial_thickness,
                        params.initial_num_points,
                    )
                }
            }
            StepType::NoStep => {}
        }
        if state.should_stich {
            stitching = stitcher::stitch(ts);
        }
        match &mut output_file {
            Some(f) => recorders::record(ts, params, f),
            None => {}
        }
    }
}

pub fn setup_renderer() -> (Renderer, Window) {
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let window: Window = WindowSettings::new("spinning-square", [800, 800])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let app = Renderer {
        gl: GlGraphics::new(opengl),
        rotation: 0.0,
    };
    (app, window)
}
