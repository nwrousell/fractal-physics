mod common;

use common::snapshot_image_bytes;
use placeholder_name::run_wfc;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_wfc_small_seed_1() {
    let output_dir = PathBuf::from("outputs/wfc");
    let output_path = output_dir.join("small_seed_1.png");
    run_wfc(1, 20, &output_path, None).unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "wfc_small_seed_1");
}

#[test]
fn test_wfc_small_seed_42() {
    let output_dir = PathBuf::from("outputs/wfc");
    let output_path = output_dir.join("small_seed_42.png");
    run_wfc(42, 20, &output_path, None).unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "wfc_small_seed_42");
}

#[test]
fn test_wfc_medium_seed_100() {
    let output_dir = PathBuf::from("outputs/wfc");
    let output_path = output_dir.join("medium_seed_100.png");
    run_wfc(100, 40, &output_path, None).unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "wfc_medium_seed_100");
}

#[test]
fn test_wfc_large_seed_999() {
    let output_dir = PathBuf::from("outputs/wfc");
    let output_path = output_dir.join("large_seed_999.png");
    run_wfc(999, 60, &output_path, None).unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "wfc_large_seed_999");
}

#[test]
fn test_wfc_knots_seed_5() {
    let output_dir = PathBuf::from("outputs/wfc");
    let output_path = output_dir.join("knots_seed_5.png");
    run_wfc(
        5,
        20,
        &output_path,
        Some("src/procgen/tilemaps/Knots/tileset.xml"),
    )
    .unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "wfc_knots_seed_5");
}

#[test]
fn test_wfc_knots_seed_123() {
    let output_dir = PathBuf::from("outputs/wfc");
    let output_path = output_dir.join("knots_seed_123.png");
    run_wfc(
        123,
        20,
        &output_path,
        Some("src/procgen/tilemaps/Knots/tileset.xml"),
    )
    .unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "wfc_knots_seed_123");
}

#[test]
fn test_wfc_circuit_seed_7() {
    let output_dir = PathBuf::from("outputs/wfc");
    let output_path = output_dir.join("circuit_seed_7.png");
    run_wfc(
        7,
        20,
        &output_path,
        Some("src/procgen/tilemaps/Circuit/tileset.xml"),
    )
    .unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "wfc_circuit_seed_7");
}

#[test]
fn test_wfc_circuit_seed_88() {
    let output_dir = PathBuf::from("outputs/wfc");
    let output_path = output_dir.join("circuit_seed_88.png");
    run_wfc(
        88,
        20,
        &output_path,
        Some("src/procgen/tilemaps/Circuit/tileset.xml"),
    )
    .unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "wfc_circuit_seed_88");
}

#[test]
fn test_wfc_castle_seed_13() {
    let output_dir = PathBuf::from("outputs/wfc");
    let output_path = output_dir.join("castle_seed_13.png");
    run_wfc(
        13,
        20,
        &output_path,
        Some("src/procgen/tilemaps/Castle/tileset.xml"),
    )
    .unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "wfc_castle_seed_13");
}

#[test]
fn test_wfc_castle_seed_256() {
    let output_dir = PathBuf::from("outputs/wfc");
    let output_path = output_dir.join("castle_seed_256.png");
    run_wfc(
        256,
        20,
        &output_path,
        Some("src/procgen/tilemaps/Castle/tileset.xml"),
    )
    .unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "wfc_castle_seed_256");
}

#[test]
fn test_wfc_floorplan_seed_9() {
    let output_dir = PathBuf::from("outputs/wfc");
    let output_path = output_dir.join("floorplan_seed_9.png");
    run_wfc(
        9,
        20,
        &output_path,
        Some("src/procgen/tilemaps/FloorPlan/tileset.xml"),
    )
    .unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "wfc_floorplan_seed_9");
}

#[test]
fn test_wfc_floorplan_seed_77() {
    let output_dir = PathBuf::from("outputs/wfc");
    let output_path = output_dir.join("floorplan_seed_77.png");
    run_wfc(
        77,
        20,
        &output_path,
        Some("src/procgen/tilemaps/FloorPlan/tileset.xml"),
    )
    .unwrap();

    let img_bytes = fs::read(&output_path).unwrap();
    snapshot_image_bytes(&img_bytes, "wfc_floorplan_seed_77");
}
