use std::collections::{HashSet, VecDeque};

use anyhow::Error;
use image::{DynamicImage, GenericImage, ImageBuffer, Rgb};
use rand::prelude::*;

use crate::procgen::types::{Tile, Tileset};

#[derive(Debug)]
pub enum WaveTile {
    Observed(usize),
    Unobserved(Vec<bool>),
}

impl WaveTile {
    fn num_possible_options(&self) -> usize {
        match self {
            Self::Observed(_) => 999999,
            Self::Unobserved(possibilities) => {
                possibilities.iter().map(|b| if *b { 1 } else { 0 }).sum()
            }
        }
    }

    #[inline]
    fn possible_options(&self) -> Vec<usize> {
        match self {
            Self::Observed(i) => vec![*i],
            Self::Unobserved(possibilities) => possibilities
                .iter()
                .enumerate()
                .filter_map(|(i, b)| if *b { Some(i) } else { None })
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct Wave {
    pub tiles: Vec<WaveTile>,
    pub width: usize,
    pub height: usize,
}

impl Wave {
    fn get(&self, x: usize, y: usize) -> &WaveTile {
        self.tiles
            .get(y * self.width + x)
            .expect("out of bounds access")
    }

    fn get_mut(&mut self, x: usize, y: usize) -> &mut WaveTile {
        self.tiles
            .get_mut(y * self.width + x)
            .expect("out of bounds access")
    }
}

pub struct WaveFunctionCollapse {
    tileset: Tileset,
    pub wave: Wave,
    rng: StdRng,
}

impl WaveFunctionCollapse {
    pub fn new(tileset: Tileset, width: usize, height: usize, seed: u64) -> Self {
        // populate WaveSlots in Unobserved state
        let mut superposition = Vec::new();
        for _ in 0..(tileset.tile_names.len() * 4) {
            superposition.push(true);
        }

        let mut tiles = Vec::new();
        for _ in 0..(width * height) {
            tiles.push(WaveTile::Unobserved(superposition.clone()));
        }

        let rng = rand::rngs::StdRng::seed_from_u64(seed);

        WaveFunctionCollapse {
            tileset,
            wave: Wave {
                tiles,
                width,
                height,
            },
            rng,
        }
    }

    pub fn step(&mut self) -> Result<(), Error> {
        // find lowest entropy tile
        let mut lowest_possibilities = self.tileset.tile_names.len() * 4;
        let mut xy = vec![(0, 0)];
        for y in 0..(self.wave.height) {
            for x in 0..(self.wave.width) {
                let possibilities = self.wave.get(x, y).num_possible_options();
                if possibilities < lowest_possibilities {
                    lowest_possibilities = possibilities;
                    xy = vec![(x, y)];
                } else if possibilities == lowest_possibilities {
                    xy.push((x, y));
                }
            }
        }

        let xy = *xy
            .choose(&mut self.rng)
            .ok_or(anyhow::anyhow!("All steps taken"))?;

        // collapse lowest entropy tile with weighted choice
        let observation = {
            let tile = self.wave.get(xy.0, xy.1);
            let possible_options = tile.possible_options();

            // Use weighted choice based on base tile weights
            *possible_options
                .choose_weighted(&mut self.rng, |&idx| {
                    let base_idx = idx / 4;
                    self.tileset.tile_weights[base_idx]
                })
                .map_err(|e| anyhow::anyhow!("Failed weighted choice: {}", e))?
        };
        let (x, y) = xy;
        *self.wave.get_mut(x, y) = WaveTile::Observed(observation);

        // propagate
        let mut propagation_queue = VecDeque::new();
        propagation_queue.push_back((x, y));
        let mut visited = HashSet::new();

        while let Some((x, y)) = propagation_queue.pop_front() {
            if visited.contains(&(x, y)) {
                continue;
            } else {
                visited.insert((x, y));
            }

            let center = self.wave.get(x, y);
            let children = self.get_children(x, y);
            let center_possible_options = center.possible_options();

            for ((child_x, child_y), side) in children {
                let child = self.wave.get(child_x, child_y);
                let disallowed_options = match child {
                    WaveTile::Unobserved(_) => {
                        let child_possible = child.possible_options();
                        let mut disallowed = Vec::new();

                        // For each child option, check if ANY center option allows it
                        'child_loop: for &child_opt in &child_possible {
                            for &center_opt in center_possible_options.iter() {
                                if self.is_allowed(center_opt, child_opt, side) {
                                    continue 'child_loop;
                                }
                            }
                            // No center tile allows this child tile - it's disallowed
                            disallowed.push(child_opt);
                        }

                        disallowed
                    }
                    WaveTile::Observed(_) => Vec::new(),
                };

                if let WaveTile::Unobserved(items) = self.wave.get_mut(child_x, child_y) {
                    for option in disallowed_options {
                        items[option] = false;
                    }

                    if !visited.contains(&(child_x, child_y)) {
                        propagation_queue.push_back((child_x, child_y));
                    }
                }
            }
        }

        Ok(())
    }

    #[inline]
    fn get_children(&self, x: usize, y: usize) -> Vec<((usize, usize), u8)> {
        let mut children = Vec::new();
        if y > 0 {
            children.push(((x, y - 1), 0));
        }
        if x > 0 {
            children.push(((x - 1, y), 1));
        }
        if y < self.wave.height - 1 {
            children.push(((x, y + 1), 2));
        }
        if x < self.wave.width - 1 {
            children.push(((x + 1, y), 3));
        }

        children
    }

    #[inline]
    fn is_allowed(&self, tile_one_idx: usize, tile_two_idx: usize, side: u8) -> bool {
        self.tileset.allowed_neighbors[tile_one_idx][side as usize][tile_two_idx]
    }

    fn index_to_tile(&self, index: usize) -> Tile {
        let base_index = index / 4;
        let rotation = index % 4;
        Tile {
            base_tile_idx: base_index,
            rotation: rotation.try_into().unwrap(),
        }
    }

    /// step until finished or in a contradictory state. Returns whether ran into a contradictory state
    pub fn step_all(&mut self) -> bool {
        for _ in 0..(self.wave.width * self.wave.height) {
            if let Err(_) = self.step() {
                return true;
            }
        }

        false
    }

    /// renders current wave to texture/image
    pub fn render(&self) -> Result<DynamicImage, Error> {
        let width = self.wave.width * self.tileset.tile_size;
        let height = self.wave.height * self.tileset.tile_size;

        let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::new(width.try_into().unwrap(), height.try_into().unwrap());

        for x in 0..self.wave.width {
            for y in 0..self.wave.height {
                let wave_slot = self.wave.get(x, y);
                match wave_slot {
                    WaveTile::Observed(tile_idx) => {
                        let tile = self.index_to_tile(*tile_idx);
                        let tile_name = &self.tileset.tile_names[tile.base_tile_idx];
                        let base_img = &self
                            .tileset
                            .tiles
                            .get(tile_name)
                            .expect("tile not found")
                            .img;
                        let rotated_img = match tile.rotation {
                            0 => base_img.clone(),
                            1 => base_img.rotate270(),
                            2 => base_img.rotate180(),
                            3 => base_img.rotate90(),
                            _ => base_img.clone(),
                        };

                        let tile_img = rotated_img.to_rgb8();

                        let px = (x * self.tileset.tile_size) as u32;
                        let py = (y * self.tileset.tile_size) as u32;

                        img.copy_from(&tile_img, px, py)?;
                    }
                    WaveTile::Unobserved(_) => (),
                }
            }
        }

        Ok(DynamicImage::from(img))
    }
}
