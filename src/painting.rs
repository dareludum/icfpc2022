use crate::block::Rect;
use crate::color::Color;
use crate::moves::Cost;
use image::{Rgba, RgbaImage};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

impl From<Color> for Rgba<u8> {
    fn from(c: Color) -> Self {
        Rgba([c.r, c.g, c.b, c.a])
    }
}

impl From<&Rgba<u8>> for Color {
    fn from(src: &Rgba<u8>) -> Self {
        Color {
            r: src[0],
            g: src[1],
            b: src[2],
            a: src[3],
        }
    }
}

#[derive(Debug)]
pub struct Painting {
    pub image: RgbaImage,
}

impl Painting {
    pub fn load(path: &std::path::Path) -> Self {
        let file = match File::open(&path) {
            Err(why) => panic!("couldn't open {:?}: {}", path, why),
            Ok(file) => file,
        };
        let mut reader = BufReader::new(file);
        let dyn_img =
            image::load(&mut reader, image::ImageFormat::Png).expect("image loading failed");
        Painting {
            image: dyn_img.into_rgba8(),
        }
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }

    pub fn get_color(&self, x: u32, y: u32) -> Color {
        self.image.get_pixel(x, y).into()
    }

    pub fn count_colors(&self, r: &Rect) -> HashMap<Color, u32> {
        let mut counts = HashMap::new();
        for x in r.x()..r.top_right.x {
            for y in r.y()..r.top_right.y {
                let color = self.get_color(x, y);
                if let Some(v) = counts.get_mut(&color) {
                    *v += 1;
                } else {
                    counts.insert(color, 1);
                }
            }
        }
        counts
    }

    pub fn calculate_score(&self, target: &Painting) -> Cost {
        if target.width() != self.width() || target.height() != self.height() {
            panic!("comparing two images different in size");
        }

        // compute the image difference score
        let image_score = self
            .image
            .pixels()
            .zip(target.image.pixels())
            .map(|(p0, p1)| {
                let mut pixel_score = 0f64;
                pixel_score += (p0.0[0].abs_diff(p1.0[0]) as f64).powi(2);
                pixel_score += (p0.0[1].abs_diff(p1.0[1]) as f64).powi(2);
                pixel_score += (p0.0[2].abs_diff(p1.0[2]) as f64).powi(2);
                pixel_score += (p0.0[3].abs_diff(p1.0[3]) as f64).powi(2);
                pixel_score.sqrt()
            })
            .sum::<f64>();
        Cost((image_score * 0.005) as u64)
    }

    pub fn write_to_file(&self, path: &std::path::Path) {
        self.image
            .save_with_format(path, image::ImageFormat::Png)
            .unwrap();
    }
}
