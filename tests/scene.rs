mod common;

use common::snapshot_image_bytes;
use core::panic;
use placeholder_name::render_scene_to_file;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_scene_small_no_png() {
    let output_dir = PathBuf::from("outputs/scene");
    let output_path = output_dir.join("small_no_png.png");
    render_scene_to_file(None, &output_path, 320, 240).unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "scene_small_no_png");
}

#[test]
fn test_scene_medium_no_png() {
    let output_dir = PathBuf::from("outputs/scene");
    let output_path = output_dir.join("medium_no_png.png");
    render_scene_to_file(None, &output_path, 640, 480).unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "scene_medium_no_png");
}

#[test]
fn test_scene_large_no_png() {
    let output_dir = PathBuf::from("outputs/scene");
    let output_path = output_dir.join("large_no_png.png");
    render_scene_to_file(None, &output_path, 800, 600).unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "scene_large_no_png");
}

#[test]
fn test_scene_with_png() {
    let png_path = "save.png";
    if !PathBuf::from(png_path).exists() {
        panic!("save.png not found, can't run test")
    }

    let output_dir = PathBuf::from("outputs/scene");
    let output_path = output_dir.join("with_png.png");
    render_scene_to_file(Some(png_path), &output_path, 320, 240).unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "scene_with_png");
}
