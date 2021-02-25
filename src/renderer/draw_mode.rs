use glutin_window::GlutinWindow as Window;
use renderer::{lines_from_thick_surface, junk};
use renderer::types::{Renderer, Line, Color};
use renderer::consts;
use piston::{Events, EventSettings, RenderEvent, MouseCursorEvent, Button, PressEvent, Event};
use graph::{cyclic_graph_from_coords, closest_node_to_some_point, distance_between_points};
use stitcher;
use graph::types::{ThickSurface, OUTER, INNER};
use linalg_helpers;

use stitcher::Stitching;

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

#[derive(Clone)]
enum State{
    Draw(Vec<(f64, f64)>, Vec<(f64, f64)>),
    SurfaceStitched(ThickSurface, Stitching),
    SurfaceUnstitched(ThickSurface),
    SurfaceStitchingA(ThickSurface, Stitching),
    SurfaceStitchingB(ThickSurface, Stitching, (usize, usize))
}

fn cyclical_lines_from_points(points: &Vec<(f64, f64)>, color: Color) -> Vec<Line> {
    let mut ret = Vec::new();
    if points.len() > 1 {
        for i in 0..points.len() {
            ret.push(Line{
                points: (points[i].0, points[i].1, points[(i+1) % points.len()].0, points[(i+1) % points.len()].1),
                color: color
            });
        }
    }
    ret
}

fn lines_around_point(x: f64, y: f64, color: Color) -> Vec<Line> {
    let circle = linalg_helpers::circular_points(x, y, 0.01, 8);
    cyclical_lines_from_points(&circle, color)
}

fn state_to_lines(s: &State, last_mouse_pos: (f64, f64)) -> Vec<Line> {
    match s {
        State::Draw(outer_points, inner_points) => {
            let mut all_lines = mk_lines(outer_points, consts::RED);
            all_lines.extend(mk_lines(inner_points, consts::BLUE).iter());
            all_lines
        }
        State::SurfaceStitched(ts, s) => {
            lines_from_thick_surface(ts, s)
        }
        State::SurfaceUnstitched(ts) => {
            lines_from_thick_surface(ts, &stitcher::Stitching::new())
        }
        State::SurfaceStitchingA(ts, s) => {
            let mut all_lines = lines_from_thick_surface(ts, s);
            let outer_n = closest_node_to_some_point(&ts.layers[OUTER], last_mouse_pos.0, last_mouse_pos.1);
            let inner_n = closest_node_to_some_point(&ts.layers[INNER], last_mouse_pos.0, last_mouse_pos.1);
            let (highlighted_x, highlighted_y) =
                if distance_between_points(last_mouse_pos.0, last_mouse_pos.1, outer_n.x, outer_n.y) <
                    distance_between_points(last_mouse_pos.0, last_mouse_pos.1, inner_n.x, inner_n.y) {
                    outer_n.pos()
                } else {
                    inner_n.pos()
                };
            all_lines.extend(lines_around_point(highlighted_x, highlighted_y, consts::WHITE));
            all_lines
        }
        State::SurfaceStitchingB(ts, s, (last_node_id, last_layer_id)) => {
            let mut all_lines = lines_from_thick_surface(ts, s);
            let outer_n = closest_node_to_some_point(&ts.layers[OUTER], last_mouse_pos.0, last_mouse_pos.1);
            let inner_n = closest_node_to_some_point(&ts.layers[INNER], last_mouse_pos.0, last_mouse_pos.1);
            let (highlighted_x, highlighted_y) =
                if distance_between_points(last_mouse_pos.0, last_mouse_pos.1, outer_n.x, outer_n.y) <
                    distance_between_points(last_mouse_pos.0, last_mouse_pos.1, inner_n.x, inner_n.y) {
                    outer_n.pos()
                } else {
                    inner_n.pos()
                };
            let mut extra_lines = lines_around_point(highlighted_x, highlighted_y, consts::WHITE);
            let (last_x, last_y) = ts.layers[*last_layer_id].nodes[*last_node_id].pos();
            extra_lines.extend(&lines_around_point(last_x, last_y, consts::TURQUOISE));
            extra_lines.extend(vec![Line{points: (last_x, last_y, highlighted_x, highlighted_y), color: consts::TURQUOISE}]);
            all_lines.extend(&extra_lines);
            all_lines
        }
        _ => Vec::new()
    }
}

fn state_effects(s: &State, e: Event, last_mouse_pos: (f64, f64)) -> State {
    match s {
        State::Draw(o, i) => {
            match e.press_args() {
                Some(Button::Mouse(piston::MouseButton::Left)) => {
                    let mut new_state_outer = o.clone();
                    new_state_outer.push(last_mouse_pos);
                    State::Draw(new_state_outer, i.clone())
                }
                Some(Button::Mouse(piston::MouseButton::Right)) => {
                    let mut new_state_inner = i.clone();
                    new_state_inner.push(last_mouse_pos);
                    State::Draw(o.clone(), new_state_inner)
                }
                Some(Button::Mouse(piston::MouseButton::Middle)) => {
                    let outer = cyclic_graph_from_coords(&o);
                    let inner = cyclic_graph_from_coords(&i);
                    let ts = ThickSurface::new(outer, inner);
                    State::SurfaceUnstitched(ts)
                }
                _ => { s.clone() }
            }
        }
        State::SurfaceUnstitched(ts) => {
            match e.press_args() {
                Some(Button::Keyboard(piston::Key::S)) => {
                    let stitch = stitcher::stitch(&ts);
                    State::SurfaceStitched(ts.clone(), stitch)
                }
                Some(Button::Mouse(piston::MouseButton::Left)) => {
                    let stitch = stitcher::Stitching::new();
                    State::SurfaceStitchingA(ts.clone(), stitch)
                }
                _ => { s.clone() }
            }
        }

        State::SurfaceStitchingA(ts, stitching) => {
            match e.press_args() {
                Some(Button::Keyboard(piston::Key::S)) => {
                    let stitch = stitcher::stitch(&ts);
                    State::SurfaceStitched(ts.clone(), stitch)
                }
                Some(Button::Mouse(piston::MouseButton::Left)) => {
                    let outer_n = closest_node_to_some_point(&ts.layers[OUTER], last_mouse_pos.0, last_mouse_pos.1);
                    let inner_n = closest_node_to_some_point(&ts.layers[INNER], last_mouse_pos.0, last_mouse_pos.1);
                    let last_ref = if distance_between_points(last_mouse_pos.0, last_mouse_pos.1, outer_n.x, outer_n.y) <
                        distance_between_points(last_mouse_pos.0, last_mouse_pos.1, inner_n.x, inner_n.y) {
                        (outer_n.id, OUTER)
                    } else {
                        (inner_n.id, INNER)
                    };
                    State::SurfaceStitchingB(ts.clone(), stitching.clone(), last_ref)
                }
                _ => { s.clone() }
            }
        }

        State::SurfaceStitchingB(ts, stitching, (last_node_id, last_layer_id)) => {
            match e.press_args() {
                Some(Button::Mouse(piston::MouseButton::Left)) => {
                    let mut stitch = stitching.clone();
                    let next_layer_id = if *last_layer_id == OUTER { INNER } else { OUTER };
                    let next_node = closest_node_to_some_point(&ts.layers[next_layer_id], last_mouse_pos.0, last_mouse_pos.1);
                    let out = (
                        if *last_layer_id == OUTER { *last_node_id } else { next_node.id },
                        if *last_layer_id == OUTER { ts.layers[*last_layer_id].nodes[*last_node_id].x } else { next_node.x },
                        if *last_layer_id == OUTER { ts.layers[*last_layer_id].nodes[*last_node_id].y } else { next_node.y },
                    );
                    let inn = (
                        if *last_layer_id == INNER { *last_node_id } else { next_node.id },
                        if *last_layer_id == INNER { ts.layers[*last_layer_id].nodes[*last_node_id].x } else { next_node.x },
                        if *last_layer_id == INNER { ts.layers[*last_layer_id].nodes[*last_node_id].y } else { next_node.y },
                    );
                    stitch.put(inn, out);
                    State::SurfaceStitchingA(ts.clone(), stitch)
                }
                _ => { s.clone() }
            }
        }

        State::SurfaceStitched(ts, _) => {
            match e.press_args() {
                Some(Button::Keyboard(piston::Key::S)) => { State::SurfaceUnstitched(ts.clone()) }
                _ => { s.clone() }
            }
        }
        _ => s.clone()
    }
}

pub fn draw_mode_rendering(
    window: &mut Window,
    renderer: &mut Renderer,
) { let mut last_mouse_pos = (0.0, 0.0);
    let mut events = Events::new(EventSettings::new());
    let mut state = State::Draw(Vec::new(), Vec::new());
    while let Some(e) = events.next(window) {
        let lines = state_to_lines(&state, last_mouse_pos);
        if let Some(args) = e.render_args() {
            renderer.render(&args, &lines);
        }

        last_mouse_pos = match e.mouse_cursor_args() {
            Some([x, y]) => {
                junk::from_window_to_minus1_1(x, y, 800.0, 800.0)
            }
            None => last_mouse_pos,
        };

        state = state_effects(&state, e, last_mouse_pos);
    }
}