mod common;

use common::snapshot_image_bytes;
use placeholder_name_lib::render_scene_to_file;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_scene() {
    let world_png = "default_world.png";
    let output_dir = PathBuf::from("outputs/scene");
    let output_path = output_dir.join("scene.png");

    render_scene_to_file(&world_png, &output_path, 320, 240, true).unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "scene_default");
}

#[test]
fn test_scene_no_postprocess() {
    let world_png = "default_world.png";
    let output_dir = PathBuf::from("outputs/scene");
    let output_path = output_dir.join("scene_no_postprocessing.png");

    render_scene_to_file(&world_png, &output_path, 320, 240, false).unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "scene_default_no_postprocessing");
}
