use winit::event_loop::EventLoop;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::app::App;
use crate::procgen::{
    WaveFunctionCollapse, WorldDefinition, bitmap_to_voxels, make_island_race_tileset,
};
use crate::scene::Scene;

mod app;
mod buffer;
mod camera;
mod game;
mod procgen;
mod scene;
mod texture;

pub fn run_wfc(seed: u64, n: usize, output_prefix: &str, make_gif: bool) -> anyhow::Result<()> {
    let img_path = output_prefix.to_owned() + ".png";
    let world_path = output_prefix.to_owned() + ".json";

    let tileset = make_island_race_tileset();
    let mut wfc = WaveFunctionCollapse::new(tileset, n, n, seed);
    let (_, bitmaps) = wfc.step_all(true, make_gif);
    if make_gif {
        let gif_path = output_prefix.to_owned() + ".gif";
        let gif_file = std::fs::File::create(&gif_path)?;
        let mut encoder = image::codecs::gif::GifEncoder::new(gif_file);
        encoder.set_repeat(image::codecs::gif::Repeat::Infinite)?;

        let frames = bitmaps.iter().map(|b| {
            let img = b.render_to_image();
            let scaled = img.resize_exact(
                img.width() * 10,
                img.height() * 10,
                image::imageops::FilterType::Nearest,
            );
            image::Frame::new(scaled.to_rgba8())
        });
        encoder.encode_frames(frames)?;
    }
    let bitmap = wfc.bitmap();
    let img = bitmap.render_to_image();
    img.save(img_path)?;

    let height_map = bitmap.compute_height_map(seed);
    let world_def = WorldDefinition { bitmap, height_map };

    let json = serde_json::to_string(&world_def)?;
    std::fs::write(world_path, json)?;

    Ok(())
}

pub fn run_interactive(
    do_postprocess: bool,
    n: usize,
    seed: u64,
    world_path: Option<&str>,
) -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop: EventLoop<crate::game::Game> = EventLoop::with_user_event().build()?;

    let voxels = if let Some(world_path) = world_path {
        let json = std::fs::read_to_string(world_path)?;
        let world_def: WorldDefinition = serde_json::from_str(&json)?;
        bitmap_to_voxels(world_def)
    } else {
        let tileset = make_island_race_tileset();
        let mut wfc = WaveFunctionCollapse::new(tileset, n, n, seed);
        wfc.step_all(true, false);
        let bitmap = wfc.bitmap();
        let height_map = bitmap.compute_height_map(seed);
        let world_def = WorldDefinition { bitmap, height_map };
        bitmap_to_voxels(world_def)
    };

    let scene = Scene::new(4, voxels);

    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
        scene,
        do_postprocess,
    );
    event_loop.run_app(&mut app)?;

    Ok(())
}

// pub fn render_scene_to_file<P: AsRef<std::path::Path>>(
//     png_path: &str,
//     output_path: P,
//     width: u32,
//     height: u32,
//     do_postprocess: bool,
// ) -> anyhow::Result<()> {
//     let (input, _, _) = generate_world_from_png(png_path)?;
//     let scene = Scene::new(4, input);

//     let mut state = pollster::block_on(Game::new_headless(scene, width, height, do_postprocess))?;
//     pollster::block_on(state.render_to_file(output_path, width, height))?;
//     Ok(())
// }

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}
