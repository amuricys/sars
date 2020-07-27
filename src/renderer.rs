use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

use types::{Graph};

pub struct Renderer {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64,  // Rotation for the square.
    colors: Vec<[f32; 4]>,
    graphs: Vec<Graph>,
}

impl Renderer {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
        let outer_color = self.colors[0];
        let inner_color = self.colors[1];

        let rotation = self.rotation;
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        let mut lines_to_draw = Vec::new();

        for g in &self.graphs {
            for edge in &g.edges {
                let from = [g.nodes[edge.source].x * args.window_size[0] / 2.0, g.nodes[edge.source].y * (- args.window_size[0] / 2.0)];
                let to = [g.nodes[edge.target].x * args.window_size[0] / 2.0, g.nodes[edge.target].y * (- args.window_size[0] / 2.0)];
                lines_to_draw.push((from, to));
            }
        }


        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            let transform = c
                .transform
                .trans(x, y)
                .rot_rad(rotation)
                .trans(-0.0, -0.0);

            for (f, t) in lines_to_draw {
                line_from_to(outer_color, 0.5, f, t, transform, gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.rotation += 2.0 * args.dt;
    }
}

pub fn setup_renderer(graphs: Vec<Graph>) -> (Window, Renderer) {
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [800, 800])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = Renderer {
        gl: GlGraphics::new(opengl),
        rotation: 0.0,
        graphs: graphs,
        colors: Vec::from([
            [1.0, 1.0, 1.0, 1.0], // white
            [1.0, 0.0, 1.0, 1.0], // purple
        ])
    };

    (window, app)
}

pub fn event_loop(renderer: &mut Renderer, window: &mut Window) {
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(window) {
        if let Some(args) = e.render_args() {
            renderer.render(&args);
        }

        if let Some(args) = e.update_args() {
            renderer.update(&args);
        }
    }
}
