use std::collections::{HashSet, VecDeque};

use anyhow::Error;
use image::{DynamicImage, ImageBuffer, Rgb};
use indicatif::{ProgressBar, ProgressStyle};
use rand::{prelude::*, rngs::StdRng};

use crate::procgen::types::{Bit, TILE_SIZE, Tile, Tileset};

#[derive(Debug)]
pub enum WaveTile {
    Observed(usize),
    Unobserved(Vec<bool>),
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Bitmap {
    pub bits: Vec<Bit>,
    pub width: usize,
    pub height: usize,
}

impl Bitmap {
    pub fn render_to_image(&self) -> DynamicImage {
        let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::new(self.width as u32, self.height as u32);

        for y in 0..self.height {
            for x in 0..self.width {
                let bit = &self.bits[y * self.width + x];
                let [r, g, b, _] = bit.color();
                let pixel = Rgb([(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]);
                img.put_pixel(x as u32, y as u32, pixel);
            }
        }

        DynamicImage::from(img)
    }

    pub fn compute_height_map(&self, seed: u64) -> HeightMap {
        let mut bottoms = vec![0; self.width * self.height];
        let mut tops = vec![1; self.width * self.height];
        let mut rng = StdRng::seed_from_u64(seed);

        // Helper to check if a bit is part of the island (not empty)
        let is_island = |bit: &Bit| *bit != Bit::Space;

        // Find all edge pixels: island pixels adjacent to empty pixels
        let mut edges: Vec<(usize, usize)> = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                if !is_island(&self.bits[idx]) {
                    continue;
                }

                // Check if any neighbor is empty (or out of bounds = edge)
                let neighbors = [
                    (x.wrapping_sub(1), y),
                    (x + 1, y),
                    (x, y.wrapping_sub(1)),
                    (x, y + 1),
                ];

                let is_edge = neighbors.iter().any(|&(nx, ny)| {
                    if nx >= self.width || ny >= self.height {
                        true // out of bounds counts as edge
                    } else {
                        !is_island(&self.bits[ny * self.width + nx])
                    }
                });

                if is_edge {
                    edges.push((x, y));
                }
            }
        }

        // BFS inward from edges, setting bottom and top values
        let mut visited = HashSet::new();
        let mut queue: VecDeque<(usize, usize, i32, i32)> = VecDeque::new(); // (x, y, bottom, top)

        // Start with bottom=-1, top=1 at edge
        for &(x, y) in &edges {
            visited.insert((x, y));
            queue.push_back((x, y, -1, 1));
            let idx = y * self.width + x;
            bottoms[idx] = -1;
            tops[idx] = 1;
        }

        // BFS with decreasing bottom values and varying top values
        while let Some((x, y, bottom, top)) = queue.pop_front() {
            let mut neighbors = [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ];
            neighbors.shuffle(&mut rng);

            for (nx, ny) in neighbors {
                if nx >= self.width || ny >= self.height {
                    continue;
                }

                let nidx = ny * self.width + nx;
                if !is_island(&self.bits[nidx]) || visited.contains(&(nx, ny)) {
                    continue;
                }

                visited.insert((nx, ny));

                // Bottom: weighted chance to go down
                let bottom_step = if rng.random_bool(0.4) { -1 } else { 0 };
                let new_bottom = bottom + bottom_step;
                bottoms[nidx] = new_bottom;

                // Top: weighted choice between -1, 0, +1, lower bounded by 1
                // Force top=1 for roads
                let new_top = if self.bits[nidx] == Bit::Road {
                    1
                } else {
                    let top_step = {
                        let r: f64 = rng.random();
                        if r < 0.33 {
                            -1
                        } else if r < 0.66 {
                            0
                        } else {
                            1
                        }
                    };
                    (top + top_step).max(1)
                };
                tops[nidx] = new_top;

                queue.push_back((nx, ny, new_bottom, new_top));
            }
        }

        HeightMap { bottoms, tops }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HeightMap {
    pub bottoms: Vec<i32>,
    pub tops: Vec<i32>,
}

impl WaveTile {
    fn num_possible_options(&self) -> usize {
        match self {
            Self::Observed(_) => usize::MAX,
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

        let mut wfc = WaveFunctionCollapse {
            tileset,
            wave: Wave {
                tiles,
                width,
                height,
            },
            rng,
        };

        // Collapse a random tile into a path end to seed the generation
        // wfc.collapse_random_to_tile("road_end");
        wfc.collapse_xy_to_tile(5, 5, "road_end");

        wfc
    }

    /// Collapse a specific tile position to a specific tile type (any rotation)
    pub fn collapse_xy_to_tile(&mut self, x: usize, y: usize, tile_name: &str) {
        // Find the base tile index for this name
        let base_idx = self
            .tileset
            .tile_names
            .iter()
            .position(|n| n == tile_name)
            .expect("tile name not found");

        // Pick a random rotation (0-3)
        let rotation = self.rng.random_range(0..4);
        let tile_idx = base_idx * 4 + rotation;

        *self.wave.get_mut(x, y) = WaveTile::Observed(tile_idx);
        self.propagate_from(x, y);
    }

    /// Collapse a random unobserved tile to a specific tile type (any rotation)
    pub fn _collapse_random_to_tile(&mut self, tile_name: &str) {
        // Find all unobserved tiles
        let unobserved: Vec<(usize, usize)> = (0..self.wave.height)
            .flat_map(|y| (0..self.wave.width).map(move |x| (x, y)))
            .filter(|&(x, y)| matches!(self.wave.get(x, y), WaveTile::Unobserved(_)))
            .collect();

        if let Some(&(x, y)) = unobserved.choose(&mut self.rng) {
            self.collapse_xy_to_tile(x, y, tile_name);
        }
    }

    /// Propagate constraints from a specific position
    fn propagate_from(&mut self, start_x: usize, start_y: usize) {
        let mut propagation_queue = VecDeque::new();
        propagation_queue.push_back((start_x, start_y));
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

                        'child_loop: for &child_opt in &child_possible {
                            for &center_opt in center_possible_options.iter() {
                                if self.is_allowed(center_opt, child_opt, side) {
                                    continue 'child_loop;
                                }
                            }
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
    pub fn step_all(&mut self, show_progress: bool, save_bitmaps: bool) -> (bool, Vec<Bitmap>) {
        let total = (self.wave.width * self.wave.height) as u64;

        let progress = if show_progress {
            let bar = ProgressBar::new(total);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            Some(bar)
        } else {
            None
        };

        let mut bitmaps = Vec::new();

        for _ in 0..total {
            if let Err(_) = self.step() {
                if let Some(bar) = progress {
                    bar.finish_with_message("contradiction!");
                }
                return (true, bitmaps);
            }
            if let Some(ref bar) = progress {
                bar.inc(1);
            }
            if save_bitmaps {
                bitmaps.push(self.bitmap());
            }
        }

        if let Some(bar) = progress {
            bar.finish_with_message("done");
        }

        (false, bitmaps)
    }

    pub fn bitmap(&self) -> Bitmap {
        let width = self.wave.width * TILE_SIZE;
        let height = self.wave.height * TILE_SIZE;

        let mut bits = vec![Bit::Empty; width * height];

        for slot_y in 0..self.wave.height {
            for slot_x in 0..self.wave.width {
                if let WaveTile::Observed(i) = self.wave.get(slot_x, slot_y) {
                    let tile = self.index_to_tile(*i);
                    let tile_name = &self.tileset.tile_names[tile.base_tile_idx];
                    let base_bitmap = &self.tileset.tiles.get(tile_name).unwrap().bitmap;
                    for tile_y in 0..TILE_SIZE {
                        let y = slot_y * TILE_SIZE + tile_y;
                        for tile_x in 0..TILE_SIZE {
                            let bit = if tile.rotation == 0 {
                                base_bitmap[tile_y][tile_x]
                            } else if tile.rotation == 1 {
                                base_bitmap[tile_x][TILE_SIZE - tile_y - 1]
                            } else if tile.rotation == 2 {
                                base_bitmap[TILE_SIZE - tile_y - 1][TILE_SIZE - tile_x - 1]
                            } else {
                                base_bitmap[TILE_SIZE - tile_x - 1][tile_y]
                            };

                            let x = slot_x * TILE_SIZE + tile_x;
                            bits[y * width + x] = bit;
                        }
                    }
                }
            }
        }

        Bitmap {
            bits,
            width,
            height,
        }
    }

    // /// renders current wave to texture/image
    // pub fn render(&self) -> Result<DynamicImage, Error> {
    //     let width = self.wave.width * self.tileset.tile_size;
    //     let height = self.wave.height * self.tileset.tile_size;

    //     let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> =
    //         ImageBuffer::new(width.try_into().unwrap(), height.try_into().unwrap());

    //     for x in 0..self.wave.width {
    //         for y in 0..self.wave.height {
    //             let wave_slot = self.wave.get(x, y);
    //             match wave_slot {
    //                 WaveTile::Observed(tile_idx) => {
    //                     let tile = self.index_to_tile(*tile_idx);
    //                     let tile_name = &self.tileset.tile_names[tile.base_tile_idx];
    //                     let base_img = &self
    //                         .tileset
    //                         .tiles
    //                         .get(tile_name)
    //                         .expect("tile not found")
    //                         .img;
    //                     let rotated_img = match tile.rotation {
    //                         0 => base_img.clone(),
    //                         1 => base_img.rotate270(),
    //                         2 => base_img.rotate180(),
    //                         3 => base_img.rotate90(),
    //                         _ => base_img.clone(),
    //                     };

    //                     let tile_img = rotated_img.to_rgb8();

    //                     let px = (x * self.tileset.tile_size) as u32;
    //                     let py = (y * self.tileset.tile_size) as u32;

    //                     img.copy_from(&tile_img, px, py)?;
    //                 }
    //                 WaveTile::Unobserved(_) => (),
    //             }
    //         }
    //     }

    //     Ok(DynamicImage::from(img))
    // }
}
