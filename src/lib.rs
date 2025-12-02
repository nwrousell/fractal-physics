use winit::event_loop::EventLoop;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::app::App;
use crate::procgen::{
    WaveFunctionCollapse, generate_world, generate_world_from_png, parse_tileset_xml,
};
use crate::state::State;

mod app;
mod camera;
mod procgen;
mod scene;
mod state;
mod texture;

pub fn render_rects_to_file<P: AsRef<std::path::Path>>(
    png_path: Option<P>,
    output_path: P,
) -> anyhow::Result<()> {
    let rects = match png_path {
        Some(path) => generate_world_from_png(path)?,
        None => generate_world()?,
    };

    // Create a 160x160 RGB image
    let mut img = image::RgbImage::new(160, 160);

    // Fill with white background
    for pixel in img.pixels_mut() {
        *pixel = image::Rgb([255, 255, 255]);
    }

    // Draw each rect with a random color
    let mut rng = rand::rng();
    use rand::Rng;

    for rect in &rects {
        // Generate random color
        let r = rng.random_range(0..=255);
        let g = rng.random_range(0..=255);
        let b = rng.random_range(0..=255);
        let color = image::Rgb([r, g, b]);

        // Convert 3D rect to 2D (top-down view using x and z coordinates)
        let x = rect.position.x as i32;
        let y = rect.position.y as i32;
        let width = (rect.width - 1.0) as i32;
        let height = (rect.height) as i32;

        // Draw the rectangle
        for dy in 0..height {
            for dx in 0..width {
                let px = x + dx;
                let py = y + dy;

                // Check bounds
                if px >= 0 && px < 160 && py >= 0 && py < 160 {
                    img.put_pixel(px as u32, py as u32, color);
                }
            }
        }
    }

    // Save the image
    img.save(output_path)?;
    println!("Saved with {} rectangles", rects.len());
    Ok(())
}

pub fn run_wfc<P: AsRef<std::path::Path>>(
    seed: u64,
    n: usize,
    output_path: P,
    tileset_path: Option<&str>,
) -> anyhow::Result<()> {
    let tileset_path = tileset_path.unwrap_or("src/procgen/tilemaps/Rooms/tileset.xml");
    let tileset = parse_tileset_xml(tileset_path)?;
    let mut wfc = WaveFunctionCollapse::new(tileset, n, n, seed);
    wfc.step_all();
    let img = wfc.render()?;
    img.save(output_path)?;
    println!("Saved WFC output with seed {} and size {}x{}", seed, n, n);
    Ok(())
}

pub fn run_interactive(png_path: Option<&str>) -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop: EventLoop<crate::state::State> = EventLoop::with_user_event().build()?;
    let png_path = png_path.map(|s| s.to_string());
    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
        png_path,
    );
    event_loop.run_app(&mut app)?;

    Ok(())
}

pub fn render_scene_to_file<P: AsRef<std::path::Path>>(
    png_path: Option<&str>,
    output_path: P,
    width: u32,
    height: u32,
) -> anyhow::Result<()> {
    let mut state = pollster::block_on(State::new_headless(png_path, width, height))?;
    pollster::block_on(state.render_to_file(output_path, width, height))?;
    Ok(())
}

pub fn run() -> anyhow::Result<()> {
    run_interactive(None)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}
