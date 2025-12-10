use std::collections::HashMap;

use crate::procgen::types::{BaseTile, Bit, TILE_SIZE, Tileset};

const R: Bit = Bit::Road;
const S: Bit = Bit::Space;
const G: Bit = Bit::Grass;
const D: Bit = Bit::Dirt;

#[rustfmt::skip]
pub const ROAD_STRAIGHT: BaseTile = BaseTile {
    bitmap: [
        [G, R, R, G], 
        [G, R, R, G], 
        [G, R, R, G], 
        [G, R, R, G]
    ],
};

#[rustfmt::skip]
pub const ROAD_TURN: BaseTile = BaseTile {
    bitmap: [
        [G, R, R, G], 
        [G, R, R, R], 
        [G, R, R, R], 
        [G, G, G, G]
    ],
};

#[rustfmt::skip]
pub const ROAD_END: BaseTile = BaseTile {
    bitmap: [
        [G, G, G, G], 
        [G, R, R, G], 
        [G, R, R, G], 
        [G, R, R, G]
    ],
};

#[rustfmt::skip]
pub const PURE_SPACE: BaseTile = BaseTile {
    bitmap: [
        [S, S, S, S], 
        [S, S, S, S], 
        [S, S, S, S], 
        [S, S, S, S]
    ],
};

#[rustfmt::skip]
pub const PURE_GRASS: BaseTile = BaseTile {
    bitmap: [
        [G, G, G, G], 
        [G, G, G, G], 
        [G, G, G, G], 
        [G, G, G, G]
    ],
};

#[rustfmt::skip]
pub const ISLAND_EDGE: BaseTile = BaseTile {
    bitmap: [
        [G, G, G, G], 
        [G, G, G, G], 
        [D, D, D, D], 
        [S, S, S, S]
    ],
};

#[rustfmt::skip]
pub const ISLAND_CORNER: BaseTile = BaseTile {
    bitmap: [
        [G, G, D, S], 
        [G, G, D, S], 
        [D, D, S, S], 
        [S, S, S, S]
    ],
};

#[rustfmt::skip]
pub const ISLAND_INNER: BaseTile = BaseTile {
    bitmap: [
        [G, G, G, G], 
        [G, G, G, G], 
        [G, G, G, D], 
        [G, G, D, S]
    ],
};

/// Each edge is read in CCW direction around the tile
fn get_edge(tile: &BaseTile, side: usize) -> Vec<Bit> {
    match side {
        0 => tile.bitmap[0].iter().rev().cloned().collect(), // top: right to left
        1 => (0..TILE_SIZE).map(|i| tile.bitmap[i][0]).collect(), // left: top to bottom
        2 => tile.bitmap[TILE_SIZE - 1].iter().cloned().collect(), // bottom: left to right
        _ => (0..TILE_SIZE)
            .rev()
            .map(|i| tile.bitmap[i][TILE_SIZE - 1])
            .collect(), // right: bottom to top
    }
}

fn edges_match(tile1: &BaseTile, side1: usize, tile2: &BaseTile, side2: usize) -> bool {
    let bits1 = get_edge(tile1, side1);
    let mut bits2 = get_edge(tile2, side2);

    bits2.reverse();

    bits1 == bits2
}

pub fn make_island_race_tileset() -> Tileset {
    // (tile, name, weight)
    let tile_defs: Vec<(BaseTile, &str, f32)> = vec![
        (ROAD_STRAIGHT, "road_straight", 1.0),
        (ROAD_TURN, "road_turn", 0.5),
        (ROAD_END, "road_end", 0.01),
        (PURE_SPACE, "pure_space", 10.0),
        (PURE_GRASS, "pure_grass", 2.0),
        (ISLAND_EDGE, "island_edge", 1.0),
        (ISLAND_CORNER, "island_corner", 1.0),
        (ISLAND_INNER, "island_inner", 1.0),
    ];

    let tiles: Vec<BaseTile> = tile_defs.iter().map(|(t, _, _)| t.clone()).collect();
    let tile_names: Vec<String> = tile_defs.iter().map(|(_, n, _)| n.to_string()).collect();
    let tile_weights: Vec<f32> = tile_defs.iter().map(|(_, _, w)| *w).collect();

    let tile_map = HashMap::from_iter(tile_names.clone().into_iter().zip(tiles.clone()));

    // produce neighbors via matching edges
    let num_tiles = tiles.len();
    let num_rotated_tiles = num_tiles * 4;
    let mut allowed_neighbors = Vec::with_capacity(num_rotated_tiles);

    for _ in 0..num_rotated_tiles {
        allowed_neighbors.push([
            vec![false; num_rotated_tiles],
            vec![false; num_rotated_tiles],
            vec![false; num_rotated_tiles],
            vec![false; num_rotated_tiles],
        ]);
    }

    for tile_one_rot_idx in 0..num_rotated_tiles {
        for side in 0..4 {
            for tile_two_rot_idx in 0..num_rotated_tiles {
                let tile_one_idx = tile_one_rot_idx / 4;
                let tile_one_rot = tile_one_rot_idx % 4;
                let tile_two_idx = tile_two_rot_idx / 4;
                let tile_two_rot = tile_two_rot_idx % 4;

                let tile1 = &tiles[tile_one_idx];
                let tile2 = &tiles[tile_two_idx];

                let side1 = (side + (4 - tile_one_rot)) % 4;
                let side2 = (side + 2 + (4 - tile_two_rot)) % 4;

                if edges_match(tile1, side1, tile2, side2) {
                    allowed_neighbors[tile_one_rot_idx][side][tile_two_rot_idx] = true;
                }
            }
        }
    }

    Tileset {
        tiles: tile_map,
        tile_names,
        allowed_neighbors,
        tile_weights,
    }
}

// make islands connected by singular path
