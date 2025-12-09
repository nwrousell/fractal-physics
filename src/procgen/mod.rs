use std::collections::VecDeque;
use std::path::Path;

use crate::scene::{RectangularPrism, SceneInput, Voxel, VoxelPos};

use anyhow::Error;
use cgmath::Point3;
use image::{DynamicImage, GenericImageView};
use rand::Rng;

mod parse;
mod types;
mod wfc;

pub use parse::parse_tileset_xml;
pub use wfc::WaveFunctionCollapse;

struct Bitmap {
    bits: Vec<bool>,
    width: usize,
    height: usize,
}

impl Bitmap {
    fn from_image(img: DynamicImage) -> Self {
        let mut bits = Vec::new();
        for y in 0..img.height() {
            for x in 0..img.width() {
                let is_black = img
                    .get_pixel(x, y)
                    .0
                    .iter()
                    .enumerate()
                    .any(|(i, c)| i < 3 && *c < 100);
                bits.push(is_black);
            }
        }

        Self {
            bits,
            width: img.width() as usize,
            height: img.height() as usize,
        }
    }

    fn get(&self, x: usize, y: usize) -> bool {
        self.bits[y * self.width + x]
    }
}

/// Flood-fill to find connected components and assign labels
fn label_connected_components(bitmap: &Bitmap) -> Vec<i32> {
    let mut labels = vec![-1i32; bitmap.width * bitmap.height];
    let mut current_label = 0;

    for start_y in 0..bitmap.height {
        for start_x in 0..bitmap.width {
            let idx = start_y * bitmap.width + start_x;
            // Skip if not black or already labeled
            if !bitmap.get(start_x, start_y) || labels[idx] >= 0 {
                continue;
            }

            // BFS flood-fill
            let mut queue = VecDeque::new();
            queue.push_back((start_x, start_y));
            labels[idx] = current_label;

            while let Some((x, y)) = queue.pop_front() {
                // Check 4-connected neighbors
                let neighbors = [
                    (x.wrapping_sub(1), y),
                    (x + 1, y),
                    (x, y.wrapping_sub(1)),
                    (x, y + 1),
                ];

                for (nx, ny) in neighbors {
                    if nx < bitmap.width && ny < bitmap.height {
                        let nidx = ny * bitmap.width + nx;
                        if bitmap.get(nx, ny) && labels[nidx] < 0 {
                            labels[nidx] = current_label;
                            queue.push_back((nx, ny));
                        }
                    }
                }
            }

            current_label += 1;
        }
    }

    labels
}

fn bitmap_to_scene_input(bitmap: &Bitmap) -> SceneInput {
    let depth = 5.0;
    let wall_thickness = 1.0;

    // Get connected component labels
    let labels = label_connected_components(bitmap);

    // Generate a color for each component
    let max_label = labels.iter().max().copied().unwrap_or(-1);
    let mut rng = rand::rng();
    let mut component_colors: Vec<[f32; 4]> = Vec::new();
    for _ in 0..=max_label {
        component_colors.push([
            rng.random_range(0.0..1.0),
            rng.random_range(0.0..1.0),
            rng.random_range(0.0..1.0),
            1.0,
        ]);
    }

    // Create voxels for the main content (these will have face culling)
    let mut voxels = Vec::new();
    for y in 0..bitmap.height {
        for x in 0..bitmap.width {
            let idx = y * bitmap.width + x;
            let label = labels[idx];
            if label >= 0 {
                let color = component_colors[label as usize];
                let voxel =
                    Voxel::new(VoxelPos::new(x as i32, y as i32, 0), 1.0, 1.0, depth, color);
                voxels.push(voxel);
            }
        }
    }

    // Create walls as prisms (no face culling - they render all faces)
    let mut prisms = Vec::new();
    let w = bitmap.width as f32;
    let h = bitmap.height as f32;
    let half_w = w / 2.0;
    let half_h = h / 2.0;
    let half_t = wall_thickness / 2.0;
    let wall_color = [0.5, 0.5, 0.5, 1.0];

    // Back wall (behind everything on z-axis)
    // prisms.push(RectangularPrism::new(
    //     Point3::new(half_w, half_h, 1.0),
    //     w + 2.0 * wall_thickness,
    //     h + 2.0 * wall_thickness,
    //     wall_thickness,
    //     wall_color,
    // ));

    // // Left wall
    // prisms.push(RectangularPrism::new(
    //     Point3::new(-half_t, half_h, 0.0),
    //     wall_thickness,
    //     h + 2.0 * wall_thickness,
    //     depth,
    //     wall_color,
    // ));

    // // Right wall
    // prisms.push(RectangularPrism::new(
    //     Point3::new(w + half_t, half_h, 0.0),
    //     wall_thickness,
    //     h + 2.0 * wall_thickness,
    //     depth,
    //     wall_color,
    // ));

    // // Top wall (y = -1)
    // prisms.push(RectangularPrism::new(
    //     Point3::new(half_w, -half_t, 0.0),
    //     w + 2.0 * wall_thickness,
    //     wall_thickness,
    //     depth,
    //     wall_color,
    // ));

    // // Bottom wall (y = h)
    // prisms.push(RectangularPrism::new(
    //     Point3::new(half_w, h + half_t, 0.0),
    //     w + 2.0 * wall_thickness,
    //     wall_thickness,
    //     depth,
    //     wall_color,
    // ));

    SceneInput { voxels, prisms }
}

pub fn generate_world_from_png<P: AsRef<Path>>(
    png_path: P,
) -> Result<(SceneInput, u32, u32), Error> {
    let img = image::open(png_path)?;
    let bitmap = Bitmap::from_image(img);
    let input = bitmap_to_scene_input(&bitmap);
    Ok((
        input,
        bitmap.width.try_into().unwrap(),
        bitmap.height.try_into().unwrap(),
    ))
}

pub fn render_rects_to_file<P: AsRef<std::path::Path>>(
    input: SceneInput,
    width: u32,
    height: u32,
    output_path: P,
) -> anyhow::Result<()> {
    let mut img = image::RgbImage::new(width, height);

    // Fill with white background
    for pixel in img.pixels_mut() {
        *pixel = image::Rgb([255, 255, 255]);
    }

    // Draw each voxel with its color
    for voxel in &input.voxels {
        let r = (voxel.color[0] * 255.0) as u8;
        let g = (voxel.color[1] * 255.0) as u8;
        let b = (voxel.color[2] * 255.0) as u8;
        let color = image::Rgb([r, g, b]);

        let x = voxel.pos.x;
        let y = voxel.pos.y;

        if x >= 0 && y >= 0 && (x as u32) < width && (y as u32) < height {
            img.put_pixel(x as u32, y as u32, color);
        }
    }

    // Save the image
    img.save(output_path)?;
    Ok(())
}
