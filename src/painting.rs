use crate::block::Color;
use crate::moves::Cost;
use image::RgbaImage;
use std::fs::File;
use std::io::BufReader;

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
        return Painting {
            image: dyn_img.into_rgba8(),
        };
    }

    pub fn width(&self) -> u32 {
        return self.image.width();
    }

    pub fn height(&self) -> u32 {
        return self.image.height();
    }

    pub fn get_color(&self, x: u32, y: u32) -> Color {
        self.image.get_pixel(x, y).into()
    }

    pub fn set_color(&mut self, x: u32, y: u32, color: &Color) {
        self.image.put_pixel(x, y, color.into());
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
        return Cost((image_score * 0.005) as u64);
    }

    pub fn write_to_file(&self, path: &std::path::Path) {
        self.image
            .save_with_format(path, image::ImageFormat::Png)
            .unwrap();
    }
}
