use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use types::{ThickSurface, OUTER, INNER, NodeChange, Node, Params, NodeChangeMap};
use simulated_annealing;
use graph_change;
use graph;
use piston::{PressEvent, Button};
use simulated_annealing::step_with_manual_change;
use recorders;
use stitcher;
use std::collections::HashMap;
use stitcher::Stitching;

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
    rotation: f64,  // Rotation for the square.
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct Line {
    points: (f64, f64, f64, f64),
    color: Color,
}

impl Renderer {
    fn render(&mut self, args: &RenderArgs, lines: &Vec<Line>) {
        use graphics::*;

        let rotation = self.rotation;
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            let transform = c
                .transform
                .trans(x, y)
                .rot_rad(rotation)
                .trans(-0.0, -0.0);

            for l in lines {
                let (x1, y1, x2, y2) = l.points;
                let col = l.color;
                let from = [x1 * args.window_size[0] / 2.0, y1 * (-args.window_size[0] / 2.0)];
                let to = [x2 * args.window_size[0] / 2.0, y2 * (-args.window_size[0] / 2.0)];
                line_from_to(col, 0.5, from, to, transform, gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
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
                points: (node.x, node.y,
                         node.next(g).x, node.next(g).y),
                color: PINK,
            });
        }
    }
    for (k, v) in &v[OUTER] {
        let outer_x = ts.layers[OUTER].nodes[*k].x;
        let outer_y = ts.layers[OUTER].nodes[*k].y;
        for val in v {
            let inner_x = ts.layers[INNER].nodes[*val].x;
            let inner_y = ts.layers[INNER].nodes[*val].y;
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
                    points: (node.x, node.y,
                             node.next(g).x, node.next(g).y),
                    color: PINK,
                });
                if i == OUTER {
                    if i == OUTER {
                        /* TODO: get lines based on acrossness map/matrix */
                    }
                }
            }
        }
    }
    for l in last_changes {
        for (_n, change) in l {
            lines.push(Line {
                points: (change.cur_x, change.cur_y,
                         change.cur_x + change.delta_x, change.cur_y + change.delta_y),
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
}

fn next_state(event: Option<Button>, s: State) -> State {
    match event {
        Some(piston::Button::Keyboard(piston::Key::Space)) => State {
            should_step: !s.should_step,
            one_at_a_time: !s.one_at_a_time,
            step_type: match s.step_type {
                StepType::Automatic => StepType::NoStep,
                _ => StepType::Automatic
            },
            ..s
        },
        Some(piston::Button::Keyboard(piston::Key::N)) => State {
            step_type: if s.one_at_a_time { StepType::OneAtATime } else { s.step_type },
            ..s
        },
        Some(piston::Button::Keyboard(piston::Key::M)) => State {
            step_type: if s.one_at_a_time { StepType::ManualChange } else { s.step_type },
            ..s
        },
        Some(piston::Button::Keyboard(piston::Key::R)) => State {
            should_step: false,
            one_at_a_time: true,
            step_type: StepType::Reset,
            temperature: 0.0,
            ..s
        },
        _ => State {
            step_type: if !s.should_step { StepType::NoStep } else { s.step_type },
            ..s
        }
    }
}

fn initial_state(initial_temperature: f64) -> State {
    State {
        should_step: false,
        one_at_a_time: true,
        step_type: StepType::NoStep,
        temperature: initial_temperature,
        should_stich: true,
    }
}

pub fn setup_optimization_and_loop<F>(ts: &mut ThickSurface,
                                      rng: &mut rand::rngs::ThreadRng,
                                      window: &mut Window,
                                      renderer: &mut Renderer,
                                      how_to_make_lines: F,
                                      params: &Params)
    where F: Fn(&ThickSurface, &Vec<NodeChangeMap>, &Stitching) -> Vec<Line> {
    let mut state = initial_state(params.initial_temperature);
    let mut stitching = stitcher::stitch(ts);
    let mut events = Events::new(EventSettings::new());
    let mut output_file = recorders::create_file_with_header("output.txt", &params.recorders);
    let mut changeset = vec![];


    while let Some(e) = events.next(window) {
        let proto_change = NodeChange { id: 0, cur_x: ts.layers[OUTER].nodes[0].x, cur_y: ts.layers[OUTER].nodes[0].y, delta_x: -0.2, delta_y: 0.0 };

        let lines = how_to_make_lines(ts, &changeset, &stitching);

        if let Some(args) = e.render_args() {
            renderer.render(&args, &lines);
        }

        if let Some(args) = e.update_args() {
            renderer.update(&args);
        }

        state = next_state(e.press_args(), state);
        match state.step_type {
            StepType::ManualChange => changeset = simulated_annealing::step_with_manual_change(ts, proto_change, params.initial_gray_matter_area, state.temperature, params, rng),
            StepType::OneAtATime => changeset = simulated_annealing::step(ts, params.initial_gray_matter_area, state.temperature, params, rng),
            StepType::Automatic => changeset = simulated_annealing::step(ts, params.initial_gray_matter_area, state.temperature, params, rng),
            StepType::Reset => *ts = {
                changeset = vec![];
                graph::circular_thick_surface(params.initial_radius, params.initial_thickness, params.initial_num_points)
            },
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

