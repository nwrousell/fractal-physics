use anyhow::{Result, bail};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum Symmetry {
    None,
    X,
    L,
    I,
    T,
    Slash,
}

impl Symmetry {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "None" => Ok(Symmetry::None),
            "X" => Ok(Symmetry::X),
            "L" => Ok(Symmetry::L),
            "I" => Ok(Symmetry::I),
            "T" => Ok(Symmetry::T),
            "\\" => Ok(Symmetry::Slash),
            "F" => Ok(Symmetry::None),
            _ => bail!("Unknown symmetry: {}", s),
        }
    }

    pub fn symmetric_sides(&self, side: u8) -> Vec<u8> {
        match self {
            Symmetry::None => vec![side],
            Symmetry::X => vec![0, 1, 2, 3],
            Symmetry::L => {
                if side == 1 || side == 2 {
                    vec![1, 2]
                } else {
                    vec![0, 3]
                }
            }
            Symmetry::I => {
                if side == 0 || side == 2 {
                    vec![0, 2]
                } else {
                    vec![1, 3]
                }
            }
            Symmetry::T => {
                if side == 1 || side == 3 {
                    vec![1, 3]
                } else {
                    vec![side]
                }
            }
            Symmetry::Slash => {
                if side == 0 || side == 1 {
                    vec![0, 1]
                } else {
                    vec![2, 3]
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct BaseTile {
    pub img: image::DynamicImage,
    pub symmetry: Symmetry,
}

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub base_tile_idx: usize,
    pub rotation: u8,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Neighbor {
    pub tile_one_idx: usize,
    pub side_one: u8,
    pub tile_two_idx: usize,
    pub side_two: u8,
}

#[derive(Debug)]
pub struct Tileset {
    pub tiles: HashMap<String, BaseTile>,
    pub tile_names: Vec<String>,
    pub tile_size: usize,
    pub allowed_neighbors: Vec<[Vec<bool>; 4]>,
    pub tile_weights: Vec<f32>,
}
