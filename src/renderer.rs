use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use types::{ThickSurface, OUTER, INNER};
use simulated_annealing;
use graph_change;
use graph;
use piston::PressEvent;

type Color = [f32; 4];
const BLACK: Color = [0.0, 0.0, 0.0, 0.0];
const WHITE: Color = [1.0, 1.0, 1.0, 1.0];
const PURPLE: Color = [0.8, 0.0, 0.8, 1.0];
const PINK: Color = [1.0, 0.4, 1.0, 1.0];
const GREEN: Color = [0.2, 1.0, 0.2, 1.0];


pub struct Renderer {
    gl: GlGraphics,
    // OpenGL drawing backend.
    rotation: f64,  // Rotation for the square.
}

#[derive (Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct Line {
    points: (f64, f64, f64, f64),
    color: Color
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

fn lines_from_thick_surface(ts: &ThickSurface) -> Vec<Line> {
    let mut lines = Vec::new();
    for i in 0..ts.layers.len() {
        let g = &ts.layers[i];
        for node in &g.nodes {
            lines.push(Line {
                points: (node.x, node.y,
                         node.next(g).x, node.next(g).y),
                color: PINK,
            });
            if i == OUTER {
                /* Non empty vector so first element is "privileged" */
                lines.push(Line { points: (node.x, node.y, ts.layers[INNER].nodes[node.acrossness[0]].x, ts.layers[INNER].nodes[node.acrossness[0]].y), color: PURPLE });
                for acr_id in 1..node.acrossness.len() - 1 {
                    lines.push(Line { points: (node.x, node.y, ts.layers[INNER].nodes[node.acrossness[acr_id]].x, ts.layers[INNER].nodes[node.acrossness[acr_id]].y), color: GREEN })
                }
            }
        }
    }
    lines
}

pub fn setup_optimization_and_loop(ts: &mut ThickSurface,
                                   rng: &mut rand::rngs::ThreadRng,
                                   window: &mut Window,
                                   renderer: &mut Renderer,
                                   initial_temperature: f64,
                                   compression_factor: f64,
                                   how_smooth: usize,
                                   node_addition_threshold: f64) {
    let initial_gray_matter_area = graph::area(&ts.layers[OUTER]) - graph::area(&ts.layers[INNER]);
    let mut should_step = false;
    let mut one_at_a_time = true;
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(window) {
        let lines = lines_from_thick_surface(ts);

        if let Some(piston::Button::Keyboard(piston::Key::Space)) = e.press_args() {
            should_step = !should_step;
            one_at_a_time = !one_at_a_time
        }

        if let Some(piston::Button::Keyboard(piston::Key::N)) = e.press_args() {
            if one_at_a_time { simulated_annealing::step(ts, initial_gray_matter_area, initial_temperature, compression_factor, how_smooth, node_addition_threshold, rng); }
        }

        if let Some(args) = e.render_args() {
            renderer.render(&args, &lines);
        }

        if let Some(args) = e.update_args() {
            renderer.update(&args);
        }

        if should_step {
            simulated_annealing::step(ts, initial_gray_matter_area, initial_temperature, compression_factor, how_smooth, node_addition_threshold, rng);
        }
    }
}

pub fn render_playground(ts: &mut ThickSurface,
                         window: &mut Window,
                         renderer: &mut Renderer,
                         which_node: usize,
                         compression_factor: f64,
                         how_smooth: usize) {

    let mut events = Events::new(EventSettings::new());
    let (outer, inner) = simulated_annealing::debug_changes(ts, how_smooth, compression_factor, which_node, (0.0, -0.2));
    let should_apply = false;
    while let Some(e) = events.next(window) {
        let lines = lines_from_thick_surface(ts);

        if let Some(key) = e.press_args() {
            if should_apply {
                graph_change::apply_changes(&mut ts.layers[OUTER], &outer);
                graph_change::apply_changes(&mut ts.layers[INNER], &inner);
            } else {
                graph_change::revert_changes(&mut ts.layers[OUTER], &outer);
                graph_change::revert_changes(&mut ts.layers[OUTER], &outer);
            }
        }

        if let Some(args) = e.render_args() {
            renderer.render(&args, &lines);
        }

        if let Some(args) = e.update_args() {
            renderer.update(&args);
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

