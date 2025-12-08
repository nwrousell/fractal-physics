use std::path::Path;

use crate::scene::RectangularPrism;

use anyhow::Error;
use cgmath::Point3;
use image::{DynamicImage, GenericImageView};

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

    fn next(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        if x < self.width - 1 {
            Some((x + 1, y))
        } else {
            if y < self.height - 1 {
                Some((0, y + 1))
            } else {
                None
            }
        }
    }

    fn first_black(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        let mut x = x;
        let mut y = y;

        loop {
            let is_black = self.get(x, y);
            if is_black {
                return Some((x, y));
            } else {
                match self.next(x, y) {
                    Some((next_x, next_y)) => {
                        x = next_x;
                        y = next_y;
                    }
                    None => {
                        return None;
                    }
                }
            }
        }
    }

    fn white_or_end(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        let mut x = x;
        let mut y = y;
        loop {
            let is_black = self.get(x, y);
            let is_last_in_row = x == self.width - 1;
            if !is_black || is_last_in_row {
                return Some((x, y));
            } else {
                match self.next(x, y) {
                    Some((next_x, next_y)) => {
                        x = next_x;
                        y = next_y;
                    }
                    None => {
                        return None;
                    }
                }
            }
        }
    }

    fn bottom(&self, left: usize, right: usize, top: usize) -> usize {
        let mut y = top + 1;
        loop {
            if y == self.height {
                return y - 1;
            }
            let row = &self.bits[(y * self.width + left)..(y * self.width + right)]; // ! here
            let all_black = row.iter().all(|b| *b);
            if all_black {
                y = y + 1;
            } else {
                return y - 1;
            }
        }
    }
}

fn bitmap_to_rects(bitmap: Bitmap) -> Vec<RectangularPrism> {
    let mut rects: Vec<RectangularPrism> = Vec::new();

    let mut x = 0;
    let mut y = 0;

    loop {
        let top_left = loop {
            match bitmap.first_black(x, y) {
                Some((first_x, first_y)) => {
                    // check if tl is included in rects
                    let first_xf = first_x as f32;
                    let first_yf = first_y as f32;
                    let mut rect_in = None;
                    for rect in &rects {
                        if first_xf >= rect.position.x
                            && first_xf < rect.position.x + rect.width
                            && first_yf >= rect.position.y
                            && first_yf < rect.position.y + rect.height
                        {
                            rect_in = Some(rect);
                            break;
                        }
                    }

                    match rect_in {
                        Some(r) => {
                            x = (r.position.x + r.width) as usize;
                            y = first_y;
                            if x >= bitmap.width {
                                x = 0;
                                y += 1;
                            }
                        }
                        None => {
                            break (first_x, first_y);
                        }
                    }
                }
                None => {
                    return rects;
                }
            }
        };

        let top_right = match bitmap.white_or_end(top_left.0, top_left.1) {
            Some(tr) => tr,
            None => unreachable!("couldn't find top right of rectangle"),
        };

        assert!(top_left.1 == top_right.1);

        let bottom = bitmap.bottom(top_left.0, top_right.0, top_left.1);

        rects.push(RectangularPrism::new(
            Point3::new(top_left.0 as f32, top_left.1 as f32, 0f32),
            (top_right.0 - top_left.0 + 1) as f32,
            (bottom - top_left.1 + 1) as f32,
            1f32,
        ));

        // continue from top-right
        match bitmap.next(top_right.0, top_right.1) {
            Some((next_x, next_y)) => {
                x = next_x;
                y = next_y;
            }
            None => {
                return rects;
            }
        }
    }
}

pub fn generate_world_from_png<P: AsRef<Path>>(
    png_path: P,
) -> Result<Vec<RectangularPrism>, Error> {
    let img = image::open(png_path)?;
    let bitmap = Bitmap::from_image(img);
    let rects = bitmap_to_rects(bitmap);
    Ok(rects)
}
