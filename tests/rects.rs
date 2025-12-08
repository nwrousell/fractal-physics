mod common;

use common::snapshot_image_bytes;
use placeholder_name_lib::render_rects_to_file;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_rects_default_world() {
    let world_png = PathBuf::from("default_world.png");
    let output_dir = PathBuf::from("outputs/rects");
    let output_path = output_dir.join("default_world.png");

    render_rects_to_file(&world_png, &output_path).unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "rects_default_world");
}
