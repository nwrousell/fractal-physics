use crate::{
    procgen::{types::Bit, wfc::Bitmap},
    scene::{Voxel, VoxelPos},
};

use anyhow::Error;
use cgmath::Point3;
use image::{DynamicImage, GenericImageView};
use rand::Rng;

mod parse;
mod tileset;
mod types;
mod wfc;

pub use tileset::make_island_race_tileset;

pub use wfc::WaveFunctionCollapse;

pub fn bitmap_to_voxels(bitmap: &Bitmap) -> Vec<Voxel> {
    let mut voxels = Vec::new();

    // TODO: height map generation

    for x in 0..bitmap.width {
        for y in 0..bitmap.height {
            let bit = bitmap.bits[y * bitmap.width + x];
            if !matches!(bit, Bit::Space) {
                let pos = VoxelPos::new(x.try_into().unwrap(), 0, y.try_into().unwrap());
                let voxel = Voxel::new(pos, 1.0, 1.0, 1.0, bit.color());
                voxels.push(voxel);
            }
        }
    }

    voxels
}

// pub fn generate_world_from_png<P: AsRef<Path>>(
//     png_path: P,
// ) -> Result<(SceneInput, u32, u32), Error> {
//     let img = image::open(png_path)?;
//     let bitmap = Bitmap::from_image(img);
//     let input = bitmap_to_scene_input(&bitmap);
//     Ok((
//         input,
//         bitmap.width.try_into().unwrap(),
//         bitmap.height.try_into().unwrap(),
//     ))
// }
