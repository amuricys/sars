mod draw_mode;
mod run_mode;

#[macro_use]
use conrod_core::*;
use rand;

use conrod_core::image::Map;
use conrod_core::position::{Align, Direction, Padding, Position, Relative};
use conrod_core::text::rt::gpu_cache::Cache;
use conrod_core::widget::file_navigator::Types::All;
use conrod_core::widget::list::Item;
use conrod_core::widget::text_box::Event;
use conrod_core::widget::Id;
use conrod_piston::event::GenericEvent;
use file_io::toml_table_to_params;
use graph::circular_thick_surface;
use graph::types::{ThickSurface, INNER, OUTER};
use my_gui::draw_mode::DrawModeAppState;
use my_gui::run_mode::RunModeAppState;
use num_traits::{Num, NumCast};
use piston_window::texture::UpdateTexture;
use piston_window::OpenGL;
use piston_window::{Flip, G2d, G2dTexture, Texture, TextureSettings};
use piston_window::{PistonWindow, UpdateEvent, Window, WindowSettings};
use regex::Regex;
use renderer::lines_from_thick_surface;
use simulated_annealing::{step, SimState};
use std::fmt::Debug;
use std::str::FromStr;
use stitcher::types::Stitching;
use types::Params;

pub const WIN_W: u32 = 1600;
pub const WIN_H: u32 = 840;

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

enum GuiMode {
    Run(RunModeAppState),
    Draw(DrawModeAppState),
}
struct App {
    mode: GuiMode,
    mouse_pos: [f64; 2],
}
impl App {
    fn new() -> App {
        App {
            mode: GuiMode::Run(RunModeAppState::new()),
            mouse_pos: [0.0, 0.0],
        }
    }
    fn handle_event(&mut self, e: &event::Input) {
        match e {
            event::Input::Motion(input::Motion::MouseCursor { x, y }) => self.mouse_pos = [*x, *y],
            _ => {}
        }
    }
}

fn handle_input_event<T>(window: &PistonWindow, event: &T, ui: &mut Ui, app: &mut App)
where
    T: GenericEvent + Clone,
{
    let size = window.size();
    let (win_w, win_h) = (size.width as conrod_core::Scalar, size.height as conrod_core::Scalar);
    if let Some(e) = conrod_piston::event::convert(event.clone(), win_w, win_h) {
        app.handle_event(&e);
        // Pass the event down to the UI so it can react and do its magic
        ui.handle_event(e);
    }
}

fn attach_gui_instance_to_ui<T>(event: &T, ui: &mut Ui, app: &mut App, run_widget_ids: &run_mode::Ids, draw_widget_ids: &draw_mode::Ids)
where
    T: GenericEvent + Clone,
{
    event.update(|_| {
        let mut ui = ui.set_widgets();
        match &mut app.mode {
            GuiMode::Draw(d) => draw_mode::gui(&mut ui, draw_widget_ids, d, app.mouse_pos),
            GuiMode::Run(r) => run_mode::gui(&mut ui, run_widget_ids, r),
        }
    });
}

fn handle_app_state(app: &mut App) {
    match &mut app.mode {
        GuiMode::Run(r) => {
            run_mode::handle_app_state(r);
            if r.is_draw_state {
                app.mode = GuiMode::Draw(DrawModeAppState::from(r.sim.clone(), r.params.clone()));
            }
        }
        GuiMode::Draw(d) => {
            draw_mode::handle_app_state(d);
            if !d.is_draw_state {
                app.mode = GuiMode::Run(RunModeAppState::from(d.sim.clone(), d.params.clone()))
            }
        }
        _ => {}
    }
}

pub fn my_ui_main() {
    const WIDTH: u32 = WIN_W;
    const HEIGHT: u32 = WIN_H;

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
    let run_mode_ids = run_mode::Ids::new(ui.widget_id_generator());
    let draw_mode_ids = draw_mode::Ids::new(ui.widget_id_generator());

    // A demonstration of some state that we'd like to control with the App.
    let mut app = App::new();
    // Poll events from the window.
    while let Some(event) = window.next() {
        // Step 1: Handle app (not gui) state
        handle_app_state(&mut app);

        // Step 2: Handle input events (this does some piston/conrod conversion, not sure how it works)
        handle_input_event(&window, &event, &mut ui, &mut app);

        // Step 3: Mutate the ui with a new instance of the gui
        attach_gui_instance_to_ui(&event, &mut ui, &mut app, &run_mode_ids, &draw_mode_ids);

        // Step 4: Draw the collected primitives to the screen
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
