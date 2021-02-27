use opengl_graphics::{GlGraphics, OpenGL};
use piston::input::{MouseCursorEvent, RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use renderer::junk::from_minus1_1_to_window;
pub type Color = [f32; 4];

pub struct Renderer {
    pub gl: GlGraphics,
    // OpenGL drawing backend.
    rotation: f64, // Rotation for the square.
}

impl Renderer {
    /* This is because we need to create a window and do some backend setup before creating the GlGraphics object.

    GitHub issue: https://github.com/PistonDevelopers/opengl_graphics/issues/103 */
    pub fn gl_ver() -> OpenGL {
        OpenGL::V3_2
    }

    pub fn new() -> Renderer {
        Renderer {
            gl: GlGraphics::new(OpenGL::V3_2),
            rotation: 0.0,
        }
    }

    pub fn render(&mut self, args: &RenderArgs, lines: &Vec<Line>) {
        use graphics::*;

        let rotation = self.rotation;
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear([0.0, 0.0, 0.0, 0.0], gl);

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

    pub fn update(&mut self, _args: &UpdateArgs) {
        // Rotate very slightly each second (0.02 radians).
        // self.rotation += 0.02 * args.dt;
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Debug)]
pub struct Line {
    pub(crate) points: (f64, f64, f64, f64),
    pub(crate) color: Color,
}
