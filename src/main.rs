mod file_io;
mod graph;
mod linalg_helpers;
mod renderer;
mod simulated_annealing;
mod stitcher;
mod types;
mod shared_shit;

extern crate float_cmp;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate pathfinding;
extern crate piston;
extern crate rand;
extern crate toml;
extern crate vec1;

extern crate conrod_core;
extern crate conrod_piston;
extern crate piston_window;
extern crate find_folder;


use renderer::draw_mode::draw_mode_rendering;
use std::env;
use stitcher::stitch_choice;


use piston_window::texture::UpdateTexture;
use piston_window::OpenGL;
use piston_window::{Flip, G2d, G2dTexture, Texture, TextureSettings};
use piston_window::{PistonWindow, UpdateEvent, Window, WindowSettings};
use conrod_core::position::{Align, Direction, Padding, Position, Relative};

pub fn conrod_main() {
    const WIDTH: u32 = shared_shit::WIN_W;
    const HEIGHT: u32 = shared_shit::WIN_H;

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("All Widgets - Piston Backend", [WIDTH, HEIGHT])
            .graphics_api(OpenGL::V3_2) // If not working, try `OpenGL::V2_1`.
            .samples(4)
            .exit_on_esc(true)
            .vsync(true)
            .build()
            .unwrap();

    // construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64])
        .theme(shared_shit::theme())
        .build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
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
        let texture =
            G2dTexture::from_memory_alpha(&mut texture_context, &init, WIDTH, HEIGHT, &settings)
                .unwrap();
        (cache, texture)
    };

    // Instantiate the generated list of widget identifiers.
    let ids = shared_shit::Ids::new(ui.widget_id_generator());

    // Load the rust logo from file to a piston_window texture.
    let rust_logo: G2dTexture = {
        let assets = find_folder::Search::ParentsThenKids(5, 3)
            .for_folder("assets")
            .unwrap();
        let path = assets.join("images/rust.png");
        let settings = TextureSettings::new();
        Texture::from_path(&mut texture_context, &path, Flip::None, &settings).unwrap()
    };

    // Create our `conrod_core::image::Map` which describes each of our widget->image mappings.
    let mut image_map = conrod_core::image::Map::new();
    let rust_logo = image_map.insert(rust_logo);

    // A demonstration of some state that we'd like to control with the App.
    let mut app = shared_shit::DemoApp::new(rust_logo);

    // Poll events from the window.
    while let Some(event) = window.next() {
        // Convert the src event to a conrod event.
        let size = window.size();
        let (win_w, win_h) = (
            size.width as conrod_core::Scalar,
            size.height as conrod_core::Scalar,
        );
        if let Some(e) = conrod_piston::event::convert(event.clone(), win_w, win_h) {
            ui.handle_event(e);
        }

        event.update(|_| {
            let mut ui = ui.set_widgets();
            shared_shit::gui(&mut ui, &ids, &mut app);
        });

        window.draw_2d(&event, |context, graphics, device| {
            if let Some(primitives) = ui.draw_if_changed() {
                // A function used for caching glyphs to the texture cache.
                let cache_queued_glyphs = |_graphics: &mut G2d,
                                           cache: &mut G2dTexture,
                                           rect: conrod_core::text::rt::Rect<u32>,
                                           data: &[u8]| {
                    let offset = [rect.min.x, rect.min.y];
                    let size = [rect.width(), rect.height()];
                    let format = piston_window::texture::Format::Rgba8;
                    text_vertex_data.clear();
                    text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));
                    UpdateTexture::update(
                        cache,
                        &mut texture_context,
                        format,
                        &text_vertex_data[..],
                        offset,
                        size,
                    )
                        .expect("failed to update texture")
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

fn real_main() {
    let params: types::Params = match std::fs::read_to_string("parameters.toml") {
        Err(_) => panic!("No parameters.toml file found in directory"),
        Ok(content) => file_io::toml_table_to_params(content.parse::<toml::Value>().unwrap()),
    };
    let mut my_graph = graph::circular_thick_surface(params.initial_radius, params.initial_thickness, params.initial_num_points);
    let mut rng = rand::thread_rng();

    let (mut renderer, mut window) = renderer::setup_renderer();

    let mut sim_state = simulated_annealing::SimState::initial_state(&params);

    renderer::setup_optimization_and_loop(
        &mut sim_state,
        &mut rng,
        &mut window,
        &mut renderer,
        |ss| renderer::lines_from_thick_surface(&ss.ts, &ss.stitching),
        &params,
    )
}

fn playin_main() {
    let (mut renderer, mut window) = renderer::setup_renderer();
    draw_mode_rendering(&mut window, &mut renderer)
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        real_main()
    } else if args[1] == "debug" {
        playin_main()
    } else if args[1] == "smart" {
        println!("Path hehe: djumba");
    } else if args[1] == "conrod" {
        conrod_main();
    }
}
