use glutin_window::GlutinWindow as Window;
use renderer::{lines_from_thick_surface, junk};
use renderer::types::{Renderer, Line, Color};
use renderer::consts::{BLUE, RED};
use piston::{Events, EventSettings, RenderEvent, MouseCursorEvent, Button, PressEvent};
use graph::{circular_thick_surface, cyclic_graph_from_coords};
use stitcher;
use graph::types::{OUTER, INNER};

fn mk_lines(points: &Vec<(f64, f64)>, color: Color) -> Vec<Line> {
    let mut lines = Vec::new();
    if points.len() >= 2 {
        for i in 0..points.len() - 1 {
            lines.push(Line { points: (points[i].0, points[i].1, points[i + 1].0, points[i + 1].1), color: color });
        }
    }
    lines
}

enum DrawModeMode {
    Outer,
    Inner,
    Surface
}

pub fn draw_mode_rendering(
    window: &mut Window,
    renderer: &mut Renderer,
) {
    let mut outer_points: Vec<(f64, f64)> = Vec::new();
    let mut inner_points: Vec<(f64, f64)> = Vec::new();
    let mut last_mouse_pos = (0.0, 0.0);
    let mut events = Events::new(EventSettings::new());
    let mut drawmodemode = DrawModeMode::Outer;
    let mut thick_surface = circular_thick_surface(0.0, 0.0, 1);
    let mut stitching = stitcher::Stitching::new();

    while let Some(e) = events.next(window) {
        let outer_lines = mk_lines(&outer_points, RED);
        let inner_lines = mk_lines(&inner_points, BLUE);
        if let Some(args) = e.render_args() {
            let mut cpy = outer_lines.clone();
            let mut cpy2 = inner_lines.clone();
            cpy.append(&mut cpy2);
            cpy = match drawmodemode {
                DrawModeMode::Surface => lines_from_thick_surface(&thick_surface, &stitching),
                _ => cpy
            };
            renderer.render(&args, &cpy);
        }

        last_mouse_pos = match e.mouse_cursor_args() {
            Some([x, y]) => {
                junk::from_window_to_minus1_1(x, y, 800.0, 800.0)
            }
            None => last_mouse_pos,
        };
        match e.press_args() {
            Some(Button::Mouse(piston::MouseButton::Left)) => {
                match drawmodemode {
                    DrawModeMode::Outer => outer_points.push(last_mouse_pos),
                    DrawModeMode::Inner => inner_points.push(last_mouse_pos),
                    _ => { }
                }
            }
            Some(Button::Mouse(piston::MouseButton::Right)) => {
                drawmodemode = match drawmodemode {
                    DrawModeMode::Outer => DrawModeMode::Inner,
                    DrawModeMode::Inner => DrawModeMode::Outer,
                    _ => DrawModeMode::Outer
                }
            }
            Some(Button::Mouse(piston::MouseButton::Middle)) => {
                thick_surface.layers[OUTER] = cyclic_graph_from_coords(&outer_points);
                thick_surface.layers[INNER] = cyclic_graph_from_coords(&inner_points);
                stitching = stitcher::stitch(&thick_surface);
                drawmodemode = DrawModeMode::Surface;
            }
            _ => {}
        }
    }
}