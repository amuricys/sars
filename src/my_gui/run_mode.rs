use conrod_core::*;

use conrod_core::widget::text_box::Event;
use conrod_core::widget::Id;
use file_io::toml_table_to_params;
use graph::types::{INNER, OUTER};
use num_traits::NumCast;
use regex::Regex;
use simulated_annealing::{step, SimState};
use std::str::FromStr;
use types::Params;
use std::collections::HashMap;
use file_io::recorders::{rec_map, RecordingState, record};
use conrod_core::position::Position::Relative;

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
pub struct RunModeAppState {
    pub(crate) params: Params,
    text_box_states: TextBoxStates,
    pub(crate) sim: SimState,
    is_paused: bool,
    pub(crate) is_draw_state: bool,
    recording_state: RecordingState,
    recorders_selection_map: HashMap<String, bool>,
    outer_color: (f32, f32, f32),
    inner_color: (f32, f32, f32),
}

impl RunModeAppState {
    pub fn new() -> Self {
        let params: Params = match std::fs::read_to_string("parameters.toml") {
            Err(_) => panic!("No parameters.toml file found in directory"),
            Ok(content) => toml_table_to_params(content.parse::<toml::Value>().unwrap()),
        };
        let mut r = HashMap::new();
        for (rn, fn_) in rec_map() {
            r.insert(rn, false);
        }
        RunModeAppState {
            sim: SimState::initial_state(&params),
            is_paused: true,
            is_draw_state: false,
            text_box_states: TextBoxStates::new(&params),
            params: params,
            outer_color: (1.0, 0.0, 1.0),
            inner_color: (0.4, 0.0, 1.0),
            recorders_selection_map: r,
            recording_state: RecordingState::empty_state("output_gui.csv").unwrap()
        }
    }
    pub fn from(ss: SimState, params: Params) -> Self {
        let mut r = HashMap::new();
        for (rn, fn_) in rec_map() {
            r.insert(rn, false);
        }
        RunModeAppState {
            sim: ss,
            is_paused: true,
            is_draw_state: false,
            text_box_states: TextBoxStates::new(&params),
            params: params,
            outer_color: (1.0, 0.0, 1.0),
            inner_color: (0.4, 0.0, 1.0),
            recorders_selection_map: r,
            recording_state: RecordingState::empty_state("output_gui.csv").unwrap()
        }
    }
}

pub fn counter_logic(lil_counter: &mut usize, lim: usize) {
    if *lil_counter > 0 {
        *lil_counter = *lil_counter + 1;
    }
    if *lil_counter > lim {
        *lil_counter = 0;
    }
}

pub fn handle_app_state(app: &mut RunModeAppState) {
    const NUM_ITERATIONS_TIL_THING_DISAPPEARS: usize = 450;

    // Step 1: Handle app (not gui) state
    if !app.is_paused {
        step(&mut app.sim, &app.params);
        record(&app.sim, &app.params, &mut app.recording_state);
    }
    counter_logic(&mut app.text_box_states.initial_thickness.1, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
    counter_logic(&mut app.text_box_states.initial_radius.1, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
    counter_logic(&mut app.text_box_states.initial_num_points.1, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
    counter_logic(&mut app.text_box_states.initial_temperature.1, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
    counter_logic(&mut app.text_box_states.compression_factor.1, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
    counter_logic(&mut app.text_box_states.softness_factor.1, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
    counter_logic(&mut app.text_box_states.how_smooth.1, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
    counter_logic(&mut app.text_box_states.max_merge_steps_away.1, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
    counter_logic(&mut app.text_box_states.node_addition_threshold.1, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
    counter_logic(&mut app.text_box_states.node_deletion_threshold.1, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
    counter_logic(&mut app.text_box_states.low.1, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
    counter_logic(&mut app.text_box_states.high.1, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
    counter_logic(&mut app.text_box_states.temperature_param.1, NUM_ITERATIONS_TIL_THING_DISAPPEARS);
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
        title_color_sliders,
        red_inner,
        green_inner,
        blue_inner,
        red_outer,
        green_outer,
        blue_outer,
        // Recorders
        title_recorders,
        energy,
        outer_perimeter,
        inner_perimeter,
        outer_area,
        inner_area,
        gray_matter_area,
        num_inner_points,
        num_outer_points,
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

<<<<<<< HEAD
fn make_recorder_widgets(anchor_id: Id, ids: &Ids, app: &mut RunModeAppState, ui: &mut conrod_core::UiCell) {
    /////////////////////////////////
    //////////////// 
    /////////////////////////////////
    const INPUT_FT_SIZE: conrod_core::FontSize = 13;

    let rec_ids: HashMap<String, Id> = vec![
        (String::from("energy"), ids.energy),
        (String::from("outer perimeter"), ids.outer_perimeter),
        (String::from("inner perimeter"), ids.inner_perimeter),
        (String::from("outer area"), ids.outer_area),
        (String::from("inner area"), ids.inner_area),
        (String::from("gray matter area"), ids.gray_matter_area),
        (String::from("num inner points"), ids.num_inner_points),
        (String::from("num outer points"), ids.num_outer_points)
    ].into_iter().collect();
    let mut p = anchor_id;
    let mut new_recorders_selection_map = app.recorders_selection_map.clone();
    for (rn, activated) in &app.recorders_selection_map {
        let n = rec_ids.get(rn).unwrap();
        for e in widget::Toggle::new(*activated)
            .label(rn)
            .label_x(conrod_core::position::Relative::Scalar(80.0))
            .color(if *activated { conrod_core::color::GREEN } else {conrod_core::color::BLACK})
            .label_font_size(INPUT_FT_SIZE)
            .down_from(p, 15.0)
            .w(30.0)
            .set(*n, ui) {
            new_recorders_selection_map.insert(rn.clone(), e);
        }
        p = *n;
    }
    app.recorders_selection_map = new_recorders_selection_map;
}


fn make_color_sliders(anchor_id: Id, ids: &Ids, app: &mut RunModeAppState, ui: &mut conrod_core::UiCell) -> Id {
    /////////////////////////////////
    //////////////// INNER PTS SLIDER
    /////////////////////////////////
    for i in widget::Slider::new(app.inner_color.0, 0.0, 1.0)
        .label("red")
        .label_color(conrod_core::color::WHITE)
        .color(Color::Rgba(app.inner_color.0, 0.0, 0.0, 1.0))
        .down_from(anchor_id, 20.0)
        .set(ids.red_inner, ui)
    {
        app.inner_color.0 = i;
    }

    for i in widget::Slider::new(app.inner_color.1, 0.0, 1.0)
        .label("green")
        .label_color(conrod_core::color::WHITE)
        .color(Color::Rgba(0.0, app.inner_color.1, 0.0, 1.0))
        .down_from(ids.red_inner, 20.0)
        .set(ids.green_inner, ui)
    {
        app.inner_color.1 = i;
    }

    for i in widget::Slider::new(app.inner_color.2, 0.0, 1.0)
        .label("blue")
        .label_color(conrod_core::color::WHITE)
        .color(Color::Rgba(0.0, 0.0, app.inner_color.2, 1.0))
        .down_from(ids.green_inner, 20.0)
        .set(ids.blue_inner, ui)
    {
        app.inner_color.2 = i;
    }
    /////////////////////////////////
    //////////////// OUTER PTS SLIDER
    /////////////////////////////////
    for i in widget::Slider::new(app.outer_color.0, 0.0, 1.0)
        .label("red")
        .label_color(conrod_core::color::WHITE)
        .color(Color::Rgba(app.outer_color.0, 0.0, 0.0, 1.0))
        .down_from(ids.blue_inner, 20.0)
        .set(ids.red_outer, ui)
    {
        app.outer_color.0 = i;
    }

    for i in widget::Slider::new(app.outer_color.1, 0.0, 1.0)
        .label("green")
        .label_color(conrod_core::color::WHITE)
        .color(Color::Rgba(0.0, app.outer_color.1, 0.0, 1.0))
        .down_from(ids.red_outer, 20.0)
        .set(ids.green_outer, ui)
    {
        app.outer_color.1 = i;
    }

    for i in widget::Slider::new(app.outer_color.2, 0.0, 1.0)
        .label("blue")
        .label_color(conrod_core::color::WHITE)
        .color(Color::Rgba(0.0, 0.0, app.outer_color.2, 1.0))
        .down_from(ids.green_outer, 20.0)
        .set(ids.blue_outer, ui)
    {
        app.outer_color.2 = i;
    }
    // Return last so other things can anchor from it
    ids.blue_outer
}

/// Instantiate a GUI demonstrating every widget available in conrod.
pub fn gui(ui: &mut conrod_core::UiCell, ids: &Ids, app: &mut RunModeAppState) {
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
        app.params.recorders = app.recorders_selection_map.iter().filter_map(|(k,v)| {if *v { Some(k.clone())} else { None }}).collect();
        app.recording_state = RecordingState::initial_state(&app.params).unwrap();
        app.sim = SimState::initial_state(&app.params);
        app.is_paused = true;
    }

    let label = if app.is_paused { "Start" } else { "Stop" };
    for _ in widget::Toggle::new(app.is_paused)
        .label(label)
        .label_color(if app.is_paused {
            conrod_core::color::WHITE
        } else {
            conrod_core::color::LIGHT_CHARCOAL
        })
        .down_from(ids.button, 20.0)
        .set(ids.toggle, ui)
    {
        app.is_paused = !app.is_paused;
    }

    // Always goes to draw state
    let label = "Draw";
    for _ in widget::Toggle::new(false)
        .label(label)
        .label_color(conrod_core::color::WHITE)
        .down_from(ids.toggle, 20.0)
        .set(ids.draw_toggle_0, ui)
    {
        app.is_draw_state = true;
    }

    widget::Text::new("Recorders")
        .down_from(ids.draw_toggle_0, 2.0)
        .set(ids.title_recorders, ui);
    let idontknow = make_recorder_widgets(ids.title_recorders, ids, app, ui);


    widget::Text::new("Outer v Inner colors")
        .right_from(ids.initial_thickness, ui.kid_area_of(ids.canvas).unwrap().w() * 0.7)
        .set(ids.title_color_sliders, ui);
    let shau = make_color_sliders(ids.title_color_sliders, ids, app, ui);


    /////////////////////////////////
    //// Actual point rendering /////
    /////////////////////////////////

    let out_pts: Vec<[f64; 2]> = app.sim.ts.points_iter(OUTER).iter().map(|n| [n.x * 400.0, n.y * 400.0]).collect();
    widget::PointPath::new(out_pts)
        .right(SHAPE_GAP)
        .color(Color::Rgba(app.outer_color.0, app.outer_color.1, app.outer_color.2, 1.0))
        .set(ids.outer_point_path, ui);
    let inn_pts: Vec<[f64; 2]> = app.sim.ts.points_iter(INNER).iter().map(|n| [n.x * 400.0, n.y * 400.0]).collect();
    widget::PointPath::new(inn_pts)
        .align_middle_x_of(ids.outer_point_path)
        .align_middle_y_of(ids.outer_point_path)
        .color(Color::Rgba(app.inner_color.0, app.inner_color.1, app.inner_color.2, 1.0))
        .set(ids.inner_point_path, ui);

    // File Navigator: It's cool
    // let file_nav_w = ui.kid_area_of(ids.canvas).unwrap().w() * 0.3;
    // let file_nav_h = ui.kid_area_of(ids.canvas).unwrap().w() * 0.3;
    // widget::FileNavigator::new(std::path::Path::new("."), All)
    //     .mid_left_with_margin_on(ids.canvas, MARGIN)
    //     .align_middle_x_of(ids.button)
    //     .w_h(file_nav_w, file_nav_h)
    //     .set(ids.file_nav, ui);
}
