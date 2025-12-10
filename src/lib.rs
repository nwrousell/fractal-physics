use winit::event_loop::EventLoop;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::app::App;
use crate::game::Game;
use crate::procgen::{WaveFunctionCollapse, bitmap_to_voxels, make_island_race_tileset};
use crate::scene::Scene;

mod app;
mod buffer;
mod camera;
mod game;
mod procgen;
mod scene;
mod texture;

pub fn run_wfc<P: AsRef<std::path::Path>>(
    seed: u64,
    n: usize,
    output_path: P,
) -> anyhow::Result<()> {
    // let tileset = parse_tileset_xml(tileset_path)?;
    let tileset = make_island_race_tileset();
    let mut wfc = WaveFunctionCollapse::new(tileset, n, n, seed);
    wfc.step_all();
    let bitmap = wfc.bitmap();
    let img = bitmap.render_to_image();
    img.save(output_path)?;
    Ok(())
}

pub fn run_interactive(do_postprocess: bool) -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop: EventLoop<crate::game::Game> = EventLoop::with_user_event().build()?;

    let tileset = make_island_race_tileset();
    let mut wfc = WaveFunctionCollapse::new(tileset, 20, 20, 17);
    wfc.step_all();
    let bitmap = wfc.bitmap();
    let voxels = bitmap_to_voxels(&bitmap);
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
