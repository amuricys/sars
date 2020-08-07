use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use types::{ThickSurface, OUTER, INNER};
use simulated_annealing;
use graph;

pub struct Renderer {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64,  // Rotation for the square.
}

impl Renderer {
    fn render(&mut self, args: &RenderArgs, lines: &Vec<(f64, f64, f64, f64)>) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
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

            for (x1, y1, x2, y2) in lines {
                line_from_to(WHITE, 0.5, [x1  * args.window_size[0] / 2.0, y1 * (- args.window_size[0] / 2.0)], [x2  * args.window_size[0] / 2.0,y2* (- args.window_size[0] / 2.0)], transform, gl);
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
            for edge in &g.edges {
                lines.push((
                    g.nodes[edge.source].x,
                    g.nodes[edge.source].y,
                    g.nodes[edge.target].x,
                    g.nodes[edge.target].y));
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
        rotation: 0.0
    };
    (app, window)
}

