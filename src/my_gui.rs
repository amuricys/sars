//! This crate is used for sharing a few items between the conrod examples.
//!
//! The module contains:
//!
//! - `pub struct DemoApp` as a demonstration of some state we want to change.
//! - `pub fn gui` as a demonstration of all widgets, some of which mutate our `DemoApp`.
//! - `pub struct Ids` - a set of all `widget::Id`s used in the `gui` fn.
//!
//! By sharing these items between these examples, we can test and ensure that the different events
//! and drawing backends behave in the same manner.
#![allow(dead_code)]

#[macro_use]
use conrod_core::*;
use rand;

use conrod_core::position::{Align, Direction, Padding, Position, Relative};
use conrod_core::widget::file_navigator::Types::All;
use conrod_core::widget::text_box::Event;
use conrod_core::widget::Id;
use file_io::toml_table_to_params;
use graph::circular_thick_surface;
use graph::types::{ThickSurface, INNER, OUTER};
use piston_window::texture::UpdateTexture;
use piston_window::OpenGL;
use piston_window::{Flip, G2d, G2dTexture, Texture, TextureSettings};
use piston_window::{PistonWindow, UpdateEvent, Window, WindowSettings};
use regex::Regex;
use renderer::lines_from_thick_surface;
use simulated_annealing::{step, SimState};
use std::str::FromStr;
use stitcher::types::Stitching;
use types::Params;
use num_traits::{Num, NumCast};
use std::fmt::Debug;

pub const WIN_W: u32 = 1600;
pub const WIN_H: u32 = 840;

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
pub struct DemoApp {
    params: Params,
    text_box_states: TextBoxStates,
    sim: SimState,
    is_paused: bool,
}

impl DemoApp {
    /// Simple constructor for the `DemoApp`.
    pub fn new() -> Self {
        let params: Params = match std::fs::read_to_string("parameters.toml") {
            Err(_) => panic!("No parameters.toml file found in directory"),
            Ok(content) => toml_table_to_params(content.parse::<toml::Value>().unwrap()),
        };
        DemoApp {
            sim: SimState::initial_state(&params),
            is_paused: true,
            text_box_states: TextBoxStates::new(&params),
            params: params,
        }
    }
}

/// A set of reasonable stylistic defaults that works for the `gui` below.
pub fn theme() -> conrod_core::Theme {
    use conrod_core::position::{Align, Direction, Padding, Position, Relative};
    conrod_core::Theme {
        name: "Demo Theme".to_string(),
        padding: Padding::none(),
        x_position: Position::Relative(Relative::Align(Align::Start), None),
        y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
        background_color: conrod_core::color::DARK_CHARCOAL,
        shape_color: conrod_core::color::LIGHT_CHARCOAL,
        border_color: conrod_core::color::BLACK,
        border_width: 0.0,
        label_color: conrod_core::color::WHITE,
        font_id: None,
        font_size_large: 26,
        font_size_medium: 18,
        font_size_small: 12,
        widget_styling: conrod_core::theme::StyleMap::default(),
        mouse_drag_threshold: 0.0,
        double_click_threshold: std::time::Duration::from_millis(500),
    }
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
        tbname_initial_thickness,
        tbname_initial_radius,
        tbname_initial_num_points,
        tbname_initial_temperature,
        tbname_initial_gray_matter_area,
        tbname_compression_factor,
        tbname_softness_factor,
        tbname_how_smooth,
        tbname_max_merge_steps_away,
        tbname_node_addition_threshold,
        tbname_node_deletion_threshold,
        tbname_low,
        tbname_high,
        tbname_recorders,
        tbname_temperature_param,
        tbname_output_file_path,
        // extra
        extra_id,
        // File navigator for deciding output
        file_nav,
        // Scrollbar
        canvas_scrollbar,
    }
}

fn counter_logic(lil_counter: &mut usize, lim: usize) {
    if *lil_counter > 0 {
        *lil_counter = *lil_counter + 1;
    }
    if *lil_counter > lim {
        *lil_counter = 0;
    }
}

fn update_param<T>(input: Event, text_box_field: &mut (String, usize), param: &mut T)
  where T: NumCast
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
)
    where T: NumCast
{
    let button_width = ui.kid_area_of(ids.canvas).unwrap().w() * 0.1;
    let button_height = ui.kid_area_of(ids.canvas).unwrap().h() * 0.05;
    const INPUT_FT_SIZE: conrod_core::FontSize = 13;
    const MARGIN: conrod_core::Scalar = 30.0;
    const SHAPE_GAP: conrod_core::Scalar = 50.0;
    const TITLE_SIZE: conrod_core::FontSize = 42;
    const SUBTITLE_SIZE: conrod_core::FontSize = 32;
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

/// Instantiate a GUI demonstrating every widget available in conrod.
pub fn gui(ui: &mut conrod_core::UiCell, ids: &Ids, app: &mut DemoApp) {
    use conrod_core::{widget, Colorable, Labelable, Positionable, Sizeable, Widget};
    use std::iter::once;

    const MARGIN: conrod_core::Scalar = 30.0;
    const SHAPE_GAP: conrod_core::Scalar = 50.0;
    const TITLE_SIZE: conrod_core::FontSize = 42;
    const SUBTITLE_SIZE: conrod_core::FontSize = 32;

    // `Canvas` is a widget that provides some basic functionality for laying out children widgets.
    // By default, its size is the size of the window. We'll use this as a background for the
    // following widgets, as well as a scrollable container for the children widgets.
    widget::Canvas::new().pad(MARGIN).scroll_kids_vertically().set(ids.canvas, ui);

    /////////////////////////////////
    /////// Input text boxes ////////
    /////////////////////////////////
    // Text box to anchor the ones below
    widget::text::Text::new("").top_left_with_margin_on(ids.canvas, MARGIN - 20.0).set(ids.anchor, ui);
    let anchor = ids.anchor;
    make_text_boxes!(
        (initial_thickness, tbname_initial_thickness, "initial thickness", app, ids, ui, anchor),
        (initial_radius, tbname_initial_radius, "initial radius", app, ids, ui, anchor),
        (initial_num_points, tbname_initial_num_points, "initial num points", app, ids, ui, anchor),
        (initial_temperature, tbname_initial_temperature, "initial temperature", app, ids, ui, anchor),
        (compression_factor, tbname_compression_factor, "compression factor", app, ids, ui, anchor),
        (softness_factor, tbname_softness_factor, "softness factor", app, ids, ui, anchor),
        (how_smooth, tbname_how_smooth, "how smooth", app, ids, ui, anchor),
        (max_merge_steps_away, tbname_max_merge_steps_away, "max merge steps away", app, ids, ui, anchor),
        (node_addition_threshold, tbname_node_addition_threshold, "node addition threshold", app, ids, ui, anchor),
        (node_deletion_threshold, tbname_node_deletion_threshold, "node deletion threshold", app, ids, ui, anchor)
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

    /////////////////////////////////
    //// Actual point rendering /////
    /////////////////////////////////

    let points: Vec<[f64; 2]> = app.sim.ts.points_iter(OUTER).iter().map(|n| [n.x * 400.0, n.y * 400.0]).collect();
    widget::PointPath::new(points).right(SHAPE_GAP).set(ids.outer_point_path, ui);
    let points2: Vec<[f64; 2]> = app.sim.ts.points_iter(INNER).iter().map(|n| [n.x * 400.0, n.y * 400.0]).collect();
    widget::PointPath::new(points2)
        .align_middle_x_of(ids.outer_point_path)
        .align_middle_y_of(ids.outer_point_path)
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

pub fn my_ui_main() {
    const WIDTH: u32 = WIN_W;
    const HEIGHT: u32 = WIN_H;
    const NUM_ITERATIONS_TIL_THING_DISAPPEARS: usize = 450;

    // Construct the window.
    let mut window: PistonWindow = WindowSettings::new("All Widgets - Piston Backend", [WIDTH, HEIGHT])
        .graphics_api(OpenGL::V3_2) // If not working, try `OpenGL::V2_1`.
        .samples(4)
        .exit_on_esc(true)
        .vsync(true)
        .build()
        .unwrap();

    // construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).theme(theme()).build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Create texture context to perform operations on textures.
    let mut texture_context = window.create_texture_context();

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_vertex_data = Vec::new();
    let (mut glyph_cache, mut text_texture_cache) = {
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;
        let cache = conrod_core::text::GlyphCache::builder()
            .dimensions(WIDTH, HEIGHT)
            .scale_tolerance(SCALE_TOLERANCE)
            .position_tolerance(POSITION_TOLERANCE)
            .build();
        let buffer_len = WIDTH as usize * HEIGHT as usize;
        let init = vec![128; buffer_len];
        let settings = TextureSettings::new();
        let texture = G2dTexture::from_memory_alpha(&mut texture_context, &init, WIDTH, HEIGHT, &settings).unwrap();
        (cache, texture)
    };

    // Stop nagging me you bastard
    let image_map = conrod_core::image::Map::new();

    // Instantiate the generated list of widget identifiers.
    let ids = Ids::new(ui.widget_id_generator());

    // A demonstration of some state that we'd like to control with the App.
    let mut app = DemoApp::new();
    // Poll events from the window.
    while let Some(event) = window.next() {
        // Disgusting state management in the while loop kkkkk
        if !app.is_paused {
            step(&mut app.sim, &app.params);
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
        // Convert the src event to a conrod event.
        let size = window.size();
        let (win_w, win_h) = (size.width as conrod_core::Scalar, size.height as conrod_core::Scalar);
        if let Some(e) = conrod_piston::event::convert(event.clone(), win_w, win_h) {
            println!("{:?} vs. {:?}", e, event);
            ui.handle_event(e);
        }

        event.update(|_| {
            let mut ui = ui.set_widgets();
            gui(&mut ui, &ids, &mut app);
        });

        window.draw_2d(&event, |context, graphics, device| {
            if let Some(primitives) = ui.draw_if_changed() {
                // A function used for caching glyphs to the texture cache.
                let cache_queued_glyphs = |_graphics: &mut G2d, cache: &mut G2dTexture, rect: conrod_core::text::rt::Rect<u32>, data: &[u8]| {
                    let offset = [rect.min.x, rect.min.y];
                    let size = [rect.width(), rect.height()];
                    let format = piston_window::texture::Format::Rgba8;
                    text_vertex_data.clear();
                    text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));
                    UpdateTexture::update(cache, &mut texture_context, format, &text_vertex_data[..], offset, size).expect("failed to update texture")
                };

                // Specify how to get the drawable texture from the image. In this case, the image
                // *is* the texture.
                fn texture_from_image<T>(img: &T) -> &T {
                    img
                }

                // Draw the conrod `render::Primitives`.
                conrod_piston::draw::primitives(
                    primitives,
                    context,
                    graphics,
                    &mut text_texture_cache,
                    &mut glyph_cache,
                    &image_map,
                    cache_queued_glyphs,
                    texture_from_image,
                );

                texture_context.encoder.flush(device);
            }
        });
    }
}
