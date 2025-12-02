mod common;

use common::snapshot_image_bytes;
use core::panic;
use placeholder_name::render_rects_to_file;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_rects_no_png() {
    let output_dir = PathBuf::from("outputs/rects");
    let output_path = output_dir.join("no_png.png");
    render_rects_to_file::<&PathBuf>(None, &output_path).unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "rects_no_png");
}

#[test]
fn test_rects_with_png() {
    let png_path = PathBuf::from("save.png");
    if !png_path.exists() {
        panic!("save.png doesn't exist, can't run test")
    }

    let output_dir = PathBuf::from("outputs/rects");
    let output_path = output_dir.join("with_png.png");
    render_rects_to_file(Some(&png_path), &output_path).unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "rects_with_png");
}
