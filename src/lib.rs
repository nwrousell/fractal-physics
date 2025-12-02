use winit::event_loop::EventLoop;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::app::App;
use crate::procgen::generate_world;

mod app;
mod camera;
mod procgen;
mod scene;
mod state;
mod texture;

pub fn generate() {
    let rects = generate_world().unwrap();

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
        let width = (rect.width) as i32;
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
    img.save("rects.png").expect("Failed to save image");
    println!("Saved rects.png with {} rectangles", rects.len());
}

pub fn run() -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    event_loop.run_app(&mut app)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}
