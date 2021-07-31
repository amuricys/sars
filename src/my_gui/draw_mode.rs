use conrod_core::*;

use conrod_core::widget::text_box::Event;
use conrod_core::widget::Id;
use file_io::toml_table_to_params;
use graph::closest_node_to_some_point;
use graph::types::{INNER, OUTER};
use linalg_helpers::{dist, lines_intersection};
use my_gui::run_mode::counter_logic;
use num_traits::NumCast;
use regex::Regex;
use simulated_annealing::SimState;
use std::str::FromStr;
use types::Params;

pub struct TextBoxStates {
    pub initial_thickness: (String, usize),
    pub initial_radius: (String, usize),
    pub initial_num_points: (String, usize),
    pub initial_temperature: (String, usize),
    pub compression_factor: (String, usize),
    pub softness_factor: (String, usize),
    pub how_smooth: (String, usize),
    pub max_merge_steps_away: (String, usize),
    pub node_addition_threshold: (String, usize),
    pub node_deletion_threshold: (String, usize),
    pub low: (String, usize),
    pub high: (String, usize),
    pub temperature_param: (String, usize),
}

impl TextBoxStates {
    fn new(params: &Params) -> TextBoxStates {
        TextBoxStates {
            initial_thickness: (params.initial_thickness.to_string(), 0),
            initial_radius: (params.initial_radius.to_string(), 0),
            initial_num_points: (params.initial_num_points.to_string(), 0),
            initial_temperature: (params.initial_temperature.to_string(), 0),
            compression_factor: (params.compression_factor.to_string(), 0),
            softness_factor: (params.softness_factor.to_string(), 0),
            how_smooth: (params.how_smooth.to_string(), 0),
            max_merge_steps_away: (params.max_merge_steps_away.to_string(), 0),
            node_addition_threshold: (params.node_addition_threshold.to_string(), 0),
            node_deletion_threshold: (params.node_deletion_threshold.to_string(), 0),
            low: (params.low_high.0.to_string(), 0),
            high: (params.low_high.1.to_string(), 0),
            temperature_param: (params.temperature_param.to_string(), 0),
        }
    }
}

/// A demonstration of some application state we want to control with a conrod GUI.
pub struct DrawMode {
    pub(crate) drawing_layers: Vec<Vec<(f64, f64)>>,
    pub(crate) attempted_intersection: usize
}

impl DrawMode {
    pub fn new() -> Self {
        DrawMode {
            drawing_layers: Vec::new(),
            attempted_intersection: 0
        }
    }
    pub fn from_inherit(ss: SimState) -> Self {
        DrawMode {
            drawing_layers: ss.ts.layers.iter().map(| g | g.to_vec_of_points()).collect(),
            attempted_intersection: 0
        }
    }
}

pub fn handle_app_state(app: &mut DrawMode, mouse_pos: &[f64; 2], just_pressed_left: bool, just_pressed_right: bool, layer_id: usize) {
    const NUM_ITERATIONS_TIL_THING_DISAPPEARS: usize = 450;

    let lines = points_to_lines(&app.drawing_layers);
    // Left tries adding
    if just_pressed_left {
        match lines_intersection(lines) {
            Some(_) => panic!("Can't add node here, would intersect at _"),
            None => app.drawing_layers[layer_id].push((mouse_pos[0], mouse_pos[1]))
        }
    }
    // Right deletes last added
    if just_pressed_right {
        let l = app.drawing_layers[layer_id].len();
        app.drawing_layers[layer_id].truncate(l - 1)
    }
    counter_logic(&mut app.attempted_intersection, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
}


// Generate a unique `WidgetId` for each widget.
widget_ids! {
    pub struct Ids {
        // The scrollable canvas.
        canvas,
        outer_point_path,
        inner_point_path,
        // Button, XyPad, Toggle.
        button,
        toggle,
        // extra
        extra_id,
        draw_toggle_0,
        draw_toggle_1,
        new_node_path,
        // Scrollbar
        canvas_scrollbar,
    }
}

pub fn gui(ui: &mut conrod_core::UiCell, ids: &Ids, app: &mut DrawMode, mouse_pos: [f64; 2]) {
    const MARGIN: conrod_core::Scalar = 30.0;
    const SHAPE_GAP: conrod_core::Scalar = 50.0;

    // `Canvas` is a widget that provides some basic functionality for laying out children widgets.
    // By default, its size is the size of the window. We'll use this as a background for the
    // following widgets, as well as a scrollable container for the children widgets.
    widget::Canvas::new().pad(MARGIN).scroll_kids_vertically().set(ids.canvas, ui);

    // Empty text box serves as anchor
    widget::text::Text::new("")
        .top_left_with_margin_on(ids.canvas, MARGIN - 20.0)
        .set(ids.anchor, ui);
    let anchor = ids.anchor;

    let button_width = ui.kid_area_of(ids.canvas).unwrap().w() * 0.12;
    let button_height = ui.kid_area_of(ids.canvas).unwrap().h() * 0.05;
    for _press in widget::Button::new()
        .label("Reset")
        .down_from(anchor, 20.0)
        .w_h(button_width, button_height)
        .set(ids.button, ui)
    {
        *app = DrawMode::new()
    }

    // Always goes to run state
    let label = "Finish";
    for _ in widget::Toggle::new(true)
        .label(label)
        .label_color(conrod_core::color::WHITE)
        .down_from(ids.button, 20.0)
        .set(ids.draw_toggle_0, ui)
    {
        app.is_draw_mode_state = false;
    }

    /////////////////////////////////
    //// Actual point rendering /////
    /////////////////////////////////

    let out_pts: Vec<[f64; 2]> = app.sim.ts.points_iter(OUTER).iter().map(|n| [n.x * 400.0, n.y * 400.0]).collect();
    widget::PointPath::new(out_pts).right(SHAPE_GAP).set(ids.outer_point_path, ui);
    let inn_pts: Vec<[f64; 2]> = app.sim.ts.points_iter(INNER).iter().map(|n| [n.x * 400.0, n.y * 400.0]).collect();
    widget::PointPath::new(inn_pts)
        .align_middle_x_of(ids.outer_point_path)
        .align_middle_y_of(ids.outer_point_path)
        .set(ids.inner_point_path, ui);
    let closest_outer = closest_node_to_some_point(&app.sim.ts.layers[OUTER], mouse_pos[0] / 400.0, mouse_pos[1] / 400.0);
    let closest_inner = closest_node_to_some_point(&app.sim.ts.layers[INNER], mouse_pos[0] / 400.0, mouse_pos[1] / 400.0);
    let to_new = if dist(closest_inner.x, closest_inner.y, mouse_pos[0] / 400.0, mouse_pos[1] / 400.0)
        < dist(closest_outer.x, closest_outer.y, mouse_pos[0] / 400.0, mouse_pos[1] / 400.0)
    {
        vec![
            [closest_inner.x * 400.0, closest_inner.y * 400.0],
            mouse_pos,
            [
                closest_inner.next(&app.sim.ts.layers[INNER]).x * 400.0,
                closest_inner.next(&app.sim.ts.layers[INNER]).y * 400.0,
            ],
        ]
    } else {
        vec![
            [closest_outer.x * 400.0, closest_outer.y * 400.0],
            mouse_pos,
            [
                closest_outer.next(&app.sim.ts.layers[OUTER]).x * 400.0,
                closest_outer.next(&app.sim.ts.layers[OUTER]).y * 400.0,
            ],
        ]
    };
    widget::PointPath::new(to_new)
        .align_middle_x_of(ids.outer_point_path)
        .align_middle_y_of(ids.outer_point_path)
        .color(conrod_core::color::PURPLE)
        .set(ids.new_node_path, ui);
    // File Navigator: It's cool
    // let file_nav_w = ui.kid_area_of(ids.canvas).unwrap().w() * 0.3;
    // let file_nav_h = ui.kid_area_of(ids.canvas).unwrap().w() * 0.3;
    // widget::FileNavigator::new(std::path::Path::new("."), All)
    //     .mid_left_with_margin_on(ids.canvas, MARGIN)
    //     .align_middle_x_of(ids.button)
    //     .w_h(file_nav_w, file_nav_h)
    //     .set(ids.file_nav, ui);
}
