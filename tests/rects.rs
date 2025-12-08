mod common;

use common::snapshot_image_bytes;
use placeholder_name_lib::parse_and_render_rects;
use std::fs;

#[test]
fn test_rects_default_world() {
    let world_png = "default_world.png";
    let output_path = "outputs/rects/default_world.png";

    parse_and_render_rects(&world_png, &output_path).unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "rects_default_world");
}
