use conrod_core::*;

use conrod_core::widget::text_box::Event;
use conrod_core::widget::Id;
use file_io::toml_table_to_params;
use graph::closest_node_to_some_point;
use graph::types::{INNER, OUTER};
use linalg_helpers::dist;
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
pub struct DrawModeAppState {
    pub(crate) params: Params,
    text_box_states: TextBoxStates,
    pub(crate) sim: SimState,
    pub(crate) is_draw_state: bool,
    pub(crate) just_added_node: usize,
    pub(crate) just_failed_to_add_node: usize,
    pub(crate) just_deleted_node: usize,
    pub(crate) just_failed_to_delete_node: usize,
}

impl DrawModeAppState {
    pub fn new() -> Self {
        let params: Params = match std::fs::read_to_string("parameters.toml") {
            Err(_) => panic!("No parameters.toml file found in directory"),
            Ok(content) => toml_table_to_params(content.parse::<toml::Value>().unwrap()),
        };
        DrawModeAppState {
            sim: SimState::initial_state(&params),
            is_draw_state: true,
            just_added_node: 0,
            just_failed_to_add_node: 0,
            just_deleted_node: 0,
            just_failed_to_delete_node: 0,
            text_box_states: TextBoxStates::new(&params),
            params: params,
        }
    }
    pub fn from(ss: SimState, params: Params) -> Self {
        DrawModeAppState {
            sim: ss,
            is_draw_state: true,
            just_added_node: 0,
            just_failed_to_add_node: 0,
            just_deleted_node: 0,
            just_failed_to_delete_node: 0,
            text_box_states: TextBoxStates::new(&params),
            params: params,
        }
    }
}

pub fn handle_app_state(app: &mut DrawModeAppState, mouse_pos: &[f64; 2], just_pressed_left: bool, just_pressed_right: bool) {
    const NUM_ITERATIONS_TIL_THING_DISAPPEARS: usize = 450;

    if just_pressed_left {
        match app.sim.ts.best_effort_add(mouse_pos[0] / 400.0, mouse_pos[1] / 400.0) {
            Ok(_) => app.just_added_node = 1,
            Err(_) => app.just_failed_to_add_node = 1,
        }
    }
    if just_pressed_right {
        match app.sim.ts.best_effort_delete(mouse_pos[0] / 400.0, mouse_pos[1] / 400.0) {
            Ok(_) => app.just_deleted_node = 1,
            Err(_) => app.just_failed_to_delete_node = 1,
        }
    }
    counter_logic(&mut app.just_added_node, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
    counter_logic(&mut app.just_failed_to_add_node, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
    counter_logic(&mut app.just_deleted_node, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
    counter_logic(&mut app.just_failed_to_delete_node, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
}

macro_rules! make_text_boxes {
    ( $(  ($param:tt, $paramname:tt, $z: expr, $app: expr, $ids: expr, $ui: expr, $anchor: tt)), *) => {
        $(
            make_text_box(
                    &mut $app.text_box_states.$param,
                    &mut $app.params.$param,
                    $anchor,
                    $ids.$param,
                    $ids.$paramname,
                    $z,
                    $ids,
                    $ui
                );
            let $anchor = $ids.$param;
        )*
    };
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
        // Text input
        anchor,
        initial_thickness,
        initial_radius,
        initial_num_points,
        initial_temperature,
        initial_gray_matter_area,
        compression_factor,
        softness_factor,
        how_smooth,
        max_merge_steps_away,
        node_addition_threshold,
        node_deletion_threshold,
        low,
        high,
        recorders,
        temperature_param,
        output_file_path,
        // Text input text
        tbninitial_thickness,
        tbninitial_radius,
        tbninitial_num_points,
        tbninitial_temperature,
        tbninitial_gray_matter_area,
        tbncompression_factor,
        tbnsoftness_factor,
        tbnhow_smooth,
        tbnmax_merge_steps_away,
        tbnnode_addition_threshold,
        tbnnode_deletion_threshold,
        tbnlow,
        tbnhigh,
        tbnrecorders,
        tbntemperature_param,
        tbnoutput_file_path,
        // extra
        extra_id,
        draw_toggle_0,
        draw_toggle_1,
        new_node_path,
        // File navigator for deciding output
        file_nav,
        // Scrollbar
        canvas_scrollbar,
    }
}

fn update_param<T>(input: Event, text_box_field: &mut (String, usize), param: &mut T)
where
    T: NumCast,
{
    match input {
        Event::Update(s) => {
            let re = Regex::new(r"^[0-9]+\.[0-9]+$").unwrap();
            if re.is_match(&*s) {
                text_box_field.0 = s;
            }
        }
        Event::Enter => {
            *param = num_traits::cast(f64::from_str(&*text_box_field.0).unwrap()).unwrap();
            text_box_field.1 = text_box_field.1 + 1; // sets off the timer until the lil prompt thing disappears
        }
    };
}

fn make_text_box<T>(
    text_box_field: &mut (String, usize),
    param: &mut T,
    anchor_id: Id,
    this_id: Id,
    this_name_id: Id,
    text: &str,
    ids: &Ids,
    ui: &mut conrod_core::UiCell,
) where
    T: NumCast,
{
    let button_width = ui.kid_area_of(ids.canvas).unwrap().w() * 0.1;
    let button_height = ui.kid_area_of(ids.canvas).unwrap().h() * 0.05;
    const INPUT_FT_SIZE: conrod_core::FontSize = 13;
    for input in widget::text_box::TextBox::new(&*text_box_field.0)
        .down_from(anchor_id, 20.0)
        .w_h(button_width, button_height)
        .set(this_id, ui)
    {
        update_param(input, text_box_field, param);
    }
    widget::text::Text::new(text)
        .right_from(this_id, 20.0)
        .font_size(INPUT_FT_SIZE)
        .set(this_name_id, ui);
    if text_box_field.1 > 0 {
        let d = ui.kid_area_of(this_name_id).unwrap().h();
        widget::text::Text::new("change will apply on next reset")
            .down_from(this_name_id, d)
            .font_size(INPUT_FT_SIZE)
            .color(color::GREEN)
            .set(ids.extra_id, ui);
    }
}

pub fn gui(ui: &mut conrod_core::UiCell, ids: &Ids, app: &mut DrawModeAppState, mouse_pos: [f64; 2]) {
    const MARGIN: conrod_core::Scalar = 30.0;
    const SHAPE_GAP: conrod_core::Scalar = 50.0;

    // `Canvas` is a widget that provides some basic functionality for laying out children widgets.
    // By default, its size is the size of the window. We'll use this as a background for the
    // following widgets, as well as a scrollable container for the children widgets.
    widget::Canvas::new().pad(MARGIN).scroll_kids_vertically().set(ids.canvas, ui);

    /////////////////////////////////
    /////// Input text boxes ////////
    /////////////////////////////////
    // Text box to anchor the ones below
    widget::text::Text::new("")
        .top_left_with_margin_on(ids.canvas, MARGIN - 20.0)
        .set(ids.anchor, ui);
    let anchor = ids.anchor;
    make_text_boxes!(
        (initial_thickness, tbninitial_thickness, "initial thickness", app, ids, ui, anchor),
        (initial_radius, tbninitial_radius, "initial radius", app, ids, ui, anchor),
        (initial_num_points, tbninitial_num_points, "initial num points", app, ids, ui, anchor),
        (initial_temperature, tbninitial_temperature, "initial temperature", app, ids, ui, anchor),
        (compression_factor, tbncompression_factor, "compression factor", app, ids, ui, anchor),
        (softness_factor, tbnsoftness_factor, "softness factor", app, ids, ui, anchor),
        (how_smooth, tbnhow_smooth, "how smooth", app, ids, ui, anchor),
        (
            max_merge_steps_away,
            tbnmax_merge_steps_away,
            "max merge steps away",
            app,
            ids,
            ui,
            anchor
        ),
        (
            node_addition_threshold,
            tbnnode_addition_threshold,
            "node addition threshold",
            app,
            ids,
            ui,
            anchor
        ),
        (
            node_deletion_threshold,
            tbnnode_deletion_threshold,
            "node deletion threshold",
            app,
            ids,
            ui,
            anchor
        )
    );
    let button_width = ui.kid_area_of(ids.canvas).unwrap().w() * 0.12;
    let button_height = ui.kid_area_of(ids.canvas).unwrap().h() * 0.05;
    for _press in widget::Button::new()
        .label("Reset")
        .down_from(anchor, 20.0)
        .w_h(button_width, button_height)
        .set(ids.button, ui)
    {
        app.sim = SimState::initial_state(&app.params);
    }

    // Always goes to run state
    let label = "Run";
    for _ in widget::Toggle::new(true)
        .label(label)
        .label_color(conrod_core::color::WHITE)
        .down_from(ids.button, 20.0)
        .set(ids.draw_toggle_0, ui)
    {
        app.is_draw_state = false;
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
