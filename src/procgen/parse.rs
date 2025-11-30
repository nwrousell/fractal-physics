use crate::procgen::types::{BaseTile, Neighbor, Symmetry, Tileset};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct TilesetDefinition {
    tiles: TilesDef,
    neighbors: NeighborsDef,
}

#[derive(Debug, Deserialize)]
struct TilesDef {
    #[serde(rename = "tile")]
    tiles: Vec<TileDef>,
}

#[derive(Debug, Deserialize)]
struct TileDef {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "@symmetry")]
    symmetry: String,
}

#[derive(Debug, Deserialize)]
struct NeighborsDef {
    #[serde(rename = "neighbor")]
    neighbors: Vec<NeighborDef>,
}

#[derive(Debug, Deserialize)]
struct NeighborDef {
    #[serde(rename = "@left")]
    left: String,
    #[serde(rename = "@right")]
    right: String,
}

/// Parses a tile reference string like "corner 1" into (tile_name, rotation)
/// where rotation is 0, 1, 2, or 3 (representing R0, R1, R2, R3)
fn parse_tile_ref(s: &str) -> (&str, u8) {
    let parts: Vec<&str> = s.trim().split_whitespace().collect();
    if parts.len() == 1 {
        (parts[0], 0)
    } else if parts.len() == 2 {
        let rotation = parts[1].parse::<u8>().unwrap_or(0);
        (parts[0], rotation)
    } else {
        (s, 0)
    }
}

/// Parse a tileset from an XML file
pub fn parse_tileset_xml<P: AsRef<Path>>(xml_path: P) -> Result<Tileset> {
    let xml_path = xml_path.as_ref();
    let xml_content = fs::read_to_string(xml_path)
        .with_context(|| format!("Failed to read XML file: {}", xml_path.display()))?;

    let def: TilesetDefinition =
        quick_xml::de::from_str(&xml_content).context("Failed to deserialize XML")?;

    // PNGs are in the same directory as the XML file
    let base_dir = xml_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    // Load tile images
    let mut tiles = HashMap::new();
    let mut tile_names = Vec::new();
    for tile_def in &def.tiles.tiles {
        let tile_path = base_dir.join(format!("{}.png", tile_def.name));
        let img = image::open(&tile_path)
            .with_context(|| format!("Failed to load tile image: {}", tile_path.display()))?;

        let symmetry = Symmetry::from_str(&tile_def.symmetry)?;
        tiles.insert(tile_def.name.clone(), BaseTile { img, symmetry });
        tile_names.push(tile_def.name.clone());
    }

    // Parse neighbors
    let mut neighbors = Vec::new();
    for neighbor_def in &def.neighbors.neighbors {
        let (name_a, rot_a) = parse_tile_ref(&neighbor_def.left);
        let (name_b, rot_b) = parse_tile_ref(&neighbor_def.right);

        let symmetry_a = tiles
            .get(name_a)
            .expect("neighbor references unknown tile")
            .symmetry;
        let symmetry_b = tiles
            .get(name_b)
            .expect("neighbor references unknown tile")
            .symmetry;

        let sides_a = symmetry_a.symmetric_sides((3 + (4 - rot_a)) % 4);
        let sides_b = symmetry_b.symmetric_sides((1 + (4 - rot_b)) % 4);

        neighbors.push(Neighbor {
            tile_one: name_a.to_string(),
            sides_one: sides_a,
            tile_two: name_b.to_string(),
            sides_two: sides_b,
        });
    }

    let tile_size = tiles.get(tile_names.first().unwrap()).unwrap().img.width() as usize;

    Ok(Tileset {
        tiles,
        tile_names,
        neighbors,
        tile_size,
    })
}
