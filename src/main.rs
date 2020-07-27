mod graph_change;
mod types;
mod graph;

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use types::NodeChange;

pub struct Renderer {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64,  // Rotation for the square.
    colors: Vec<[f32; 4]>,
    graphs: Vec<types::Graph>,
}

impl Renderer {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
        let outer_color = self.colors[0];
        // let inner_color = self.colors[1];

        let rotation = self.rotation;
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        let mut lines_to_draw = Vec::new();

        for g in &self.graphs {
            for edge in &self.graphs[0].edges {
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

fn main() {
    let opengl = OpenGL::V3_2;

    let mut my_graph = graph::thick_surface(1.0, 0.2,  100);
    let my_change = NodeChange {id: 0, cur_x: my_graph.outer.nodes[0].x, cur_y: my_graph.outer.nodes[0].y, new_x: my_graph.outer.nodes[0].x - 0.2, new_y: my_graph.outer.nodes[0].y};
    let changeset = graph_change::smooth_change_out(&my_graph.outer, my_change, 3.5);
    graph_change::apply_changes(&mut my_graph.outer, changeset);

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
        graphs: Vec::from([my_graph.outer, my_graph.inner]),
        colors: Vec::from([[1.0, 1.0, 1.0, 1.0]])
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}
