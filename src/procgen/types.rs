use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BaseTile {
    pub bitmap: TileBitmap,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Bit {
    Road,
    Space,
    Grass,
    Dirt,
    Empty,
}

impl Bit {
    pub fn color(&self) -> [f32; 4] {
        match self {
            Bit::Road => [0.3, 0.3, 0.3, 1.0],    // dark gray
            Bit::Space => [0.9, 0.9, 0.95, 1.0],  // near white/light blue
            Bit::Grass => [0.2, 0.7, 0.2, 1.0],   // green
            Bit::Dirt => [0.55, 0.27, 0.07, 1.0], // brown
            Bit::Empty => [0.0, 0.0, 0.0, 1.0],
        }
    }
}

pub struct Tile {
    pub base_tile_idx: usize,
    pub rotation: u8,
}

pub const TILE_SIZE: usize = 4;

pub type TileBitmap = [[Bit; TILE_SIZE]; TILE_SIZE];

#[derive(Debug)]
pub struct Tileset {
    pub tiles: HashMap<String, BaseTile>,
    pub tile_names: Vec<String>,
    pub allowed_neighbors: Vec<[Vec<bool>; 4]>,
    pub tile_weights: Vec<f32>,
}
