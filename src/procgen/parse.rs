// use crate::procgen::types::{BaseTile, Neighbor, Symmetry, Tileset};
// use anyhow::{Context, Result};
// use serde::Deserialize;
// use std::collections::{HashMap, HashSet};
// use std::fs;
// use std::path::Path;

// #[derive(Debug, Deserialize)]
// struct TilesetDefinition {
//     tiles: TilesDef,
//     neighbors: NeighborsDef,
// }

// #[derive(Debug, Deserialize)]
// struct TilesDef {
//     #[serde(rename = "tile")]
//     tiles: Vec<TileDef>,
// }

// #[derive(Debug, Deserialize)]
// struct TileDef {
//     #[serde(rename = "@name")]
//     name: String,
//     #[serde(rename = "@symmetry")]
//     symmetry: String,
//     #[serde(rename = "@weight", default = "default_weight")]
//     weight: f32,
// }

// fn default_weight() -> f32 {
//     1.0
// }

// #[derive(Debug, Deserialize)]
// struct NeighborsDef {
//     #[serde(rename = "neighbor")]
//     neighbors: Vec<NeighborDef>,
// }

// #[derive(Debug, Deserialize)]
// struct NeighborDef {
//     #[serde(rename = "@left")]
//     left: String,
//     #[serde(rename = "@right")]
//     right: String,
// }

// /// Parses a tile reference string like "corner 1" into (tile_name, rotation)
// /// where rotation is 0, 1, 2, or 3 (representing R0, R1, R2, R3)
// fn parse_tile_ref(s: &str) -> (&str, u8) {
//     let parts: Vec<&str> = s.trim().split_whitespace().collect();
//     if parts.len() == 1 {
//         (parts[0], 0)
//     } else if parts.len() == 2 {
//         let rotation = parts[1].parse::<u8>().unwrap_or(0);
//         (parts[0], rotation)
//     } else {
//         (s, 0)
//     }
// }

// /// Parse a tileset from an XML file
// pub fn parse_tileset_xml<P: AsRef<Path>>(xml_path: P) -> Result<Tileset> {
//     let xml_path = xml_path.as_ref();
//     let xml_content = fs::read_to_string(xml_path)
//         .with_context(|| format!("Failed to read XML file: {}", xml_path.display()))?;

//     let def: TilesetDefinition =
//         quick_xml::de::from_str(&xml_content).context("Failed to deserialize XML")?;

//     // PNGs are in the same directory as the XML file
//     let base_dir = xml_path
//         .parent()
//         .unwrap_or_else(|| Path::new("."))
//         .to_path_buf();

//     // Load tile images
//     let mut tiles = HashMap::new();
//     let mut tile_names = Vec::new();
//     let mut tile_weights = Vec::new();
//     let mut name_to_index = HashMap::new();
//     for (idx, tile_def) in def.tiles.tiles.iter().enumerate() {
//         let tile_path = base_dir.join(format!("{}.png", tile_def.name));
//         let img = image::open(&tile_path)
//             .with_context(|| format!("Failed to load tile image: {}", tile_path.display()))?;

//         let symmetry = Symmetry::from_str(&tile_def.symmetry)?;
//         tiles.insert(tile_def.name.clone(), BaseTile { img, symmetry });
//         tile_names.push(tile_def.name.clone());
//         tile_weights.push(tile_def.weight);
//         name_to_index.insert(tile_def.name.as_str(), idx);
//     }

//     // Parse neighbors
//     let mut neighbors = HashSet::new();
//     for neighbor_def in &def.neighbors.neighbors {
//         let (name_a, rot_a) = parse_tile_ref(&neighbor_def.left);
//         let (name_b, rot_b) = parse_tile_ref(&neighbor_def.right);

//         let idx_a = *name_to_index
//             .get(name_a)
//             .expect("neighbor references unknown tile");
//         let idx_b = *name_to_index
//             .get(name_b)
//             .expect("neighbor references unknown tile");

//         let symmetry_a = tiles
//             .get(name_a)
//             .expect("neighbor references unknown tile")
//             .symmetry;
//         let symmetry_b = tiles
//             .get(name_b)
//             .expect("neighbor references unknown tile")
//             .symmetry;

//         let sides_a = symmetry_a.symmetric_sides((3 + (4 - rot_a)) % 4);
//         let sides_b = symmetry_b.symmetric_sides((1 + (4 - rot_b)) % 4);

//         // Expand into individual side-to-side neighbors
//         for &side_a in &sides_a {
//             for &side_b in &sides_b {
//                 neighbors.insert(Neighbor {
//                     tile_one_idx: idx_a,
//                     side_one: side_a,
//                     tile_two_idx: idx_b,
//                     side_two: side_b,
//                 });
//             }
//         }
//     }

//     // let tile_size = tiles.get(tile_names.first().unwrap()).unwrap().img.width() as usize;

//     // Pre-compute allowed neighbors
//     let num_tiles = tile_names.len();
//     let num_rotated_tiles = num_tiles * 4;
//     let mut allowed_neighbors = Vec::with_capacity(num_rotated_tiles);

//     for _ in 0..num_rotated_tiles {
//         allowed_neighbors.push([
//             vec![false; num_rotated_tiles],
//             vec![false; num_rotated_tiles],
//             vec![false; num_rotated_tiles],
//             vec![false; num_rotated_tiles],
//         ]);
//     }

//     // For each neighbor relationship, mark all compatible rotations
//     for neighbor in &neighbors {
//         // Try all rotation combinations
//         for rot_one in 0..4u8 {
//             for rot_two in 0..4u8 {
//                 let tile_one_idx = neighbor.tile_one_idx * 4 + rot_one as usize;
//                 let tile_two_idx = neighbor.tile_two_idx * 4 + rot_two as usize;

//                 for side in 0..4u8 {
//                     let side_one = (side + (4 - rot_one)) % 4;
//                     let side_two = (side + 2 + (4 - rot_two)) % 4;

//                     if neighbor.side_one == side_one && neighbor.side_two == side_two {
//                         allowed_neighbors[tile_one_idx][side as usize][tile_two_idx] = true;
//                     }

//                     // Also check the other way
//                     let side_one_rev = (side + (4 - rot_two)) % 4;
//                     let side_two_rev = (side + 2 + (4 - rot_one)) % 4;

//                     if neighbor.side_two == side_one_rev && neighbor.side_one == side_two_rev {
//                         allowed_neighbors[tile_two_idx][side as usize][tile_one_idx] = true;
//                     }
//                 }
//             }
//         }
//     }

//     Ok(Tileset {
//         tiles,
//         tile_names,
//         tile_size,
//         allowed_neighbors,
//         tile_weights,
//     })
// }
