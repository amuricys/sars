use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use types::{ThickSurface, OUTER, INNER};
use simulated_annealing;
use graph;

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
                line_from_to(col, 0.5, [x1 * args.window_size[0] / 2.0, y1 * (-args.window_size[0] / 2.0)], [x2 * args.window_size[0] / 2.0, y2 * (-args.window_size[0] / 2.0)], transform, gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.rotation += 2.0 * args.dt;
    }
}

pub fn setup_optimization_and_loop(ts: &mut ThickSurface,
                                   rng: &mut rand::rngs::ThreadRng,
                                   window: &mut Window,
                                   renderer: &mut Renderer,
                                   initial_temperature: f64,
                                   compression_factor: f64,
                                   how_smooth: usize) {
    let initial_gray_matter_area = graph::area(&ts.layers[OUTER]) - graph::area(&ts.layers[INNER]);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(window) {
        let mut lines = Vec::new();
        for i in 0..ts.layers.len() {
            let g = &ts.layers[i];
            for node in &g.nodes {
                lines.push(Line {
                    points: (node.x, node.y,
                             node.next(g).x, node.next(g).y),
                    color: PINK,
                });
                if i == 0 {
                    if let Some(x) = node.acrossness.mid {
                        lines.push(Line { points: (node.x, node.y, ts.layers[1].nodes[x].x, ts.layers[1].nodes[x].y), color: PURPLE })
                    }
                    if let Some(x) = node.acrossness.prev {
                        lines.push(Line { points: (node.x, node.y, ts.layers[1].nodes[x].x, ts.layers[1].nodes[x].y), color: GREEN })
                    }
                    if let Some(x) = node.acrossness.next {
                        lines.push(Line { points: (node.x, node.y, ts.layers[1].nodes[x].x, ts.layers[1].nodes[x].y), color: GREEN })
                    }
                }
            }
        }

        if let Some(args) = e.render_args() {
            renderer.render(&args, &lines);
        }

        if let Some(args) = e.update_args() {
            renderer.update(&args);
        }

        simulated_annealing::step(ts, initial_gray_matter_area, initial_temperature, compression_factor, how_smooth, rng);
    }
}

pub fn render_line_list(lines: &Vec<Line>,
                        window: &mut Window,
                        renderer: &mut Renderer) {

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(window) {
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

