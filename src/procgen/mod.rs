use crate::{
    procgen::{
        types::Bit,
        wfc::{Bitmap, HeightMap},
    },
    scene::{Voxel, VoxelPos},
};

mod parse;
mod tileset;
mod types;
mod wfc;

pub use tileset::make_island_race_tileset;

pub use wfc::WaveFunctionCollapse;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct WorldDefinition {
    pub bitmap: Bitmap,
    pub height_map: HeightMap,
}

pub fn bitmap_to_voxels(world_def: WorldDefinition) -> Vec<Voxel> {
    let mut voxels = Vec::new();

    let bitmap = world_def.bitmap;
    let height_map = world_def.height_map;

    for x in 0..bitmap.width {
        for y in 0..bitmap.height {
            let bit = bitmap.bits[y * bitmap.width + x];
            if !matches!(bit, Bit::Space) {
                let bottom = height_map.bottoms[y * bitmap.width + x];
                let top = height_map.tops[y * bitmap.width + x];

                for level in bottom..0 {
                    let pos = VoxelPos::new(x.try_into().unwrap(), level, y.try_into().unwrap());
                    let voxel = Voxel::new(pos, 1.0, 1.0, 1.0, Bit::Dirt.color());
                    voxels.push(voxel);
                }

                for level in 0..top {
                    let pos = VoxelPos::new(x.try_into().unwrap(), level, y.try_into().unwrap());
                    let voxel = Voxel::new(pos, 1.0, 1.0, 1.0, bit.color());
                    voxels.push(voxel);
                }
            }
        }
    }

    voxels
}
