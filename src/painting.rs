use crate::block::Color;
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

    pub fn calculate_score(&self, target: &Painting) -> f32 {
        if target.width() != self.width() || target.height() != self.height() {
            panic!("comparing two images different in size");
        }

        // compute the image difference score
        let image_score = self
            .image
            .pixels()
            .zip(target.image.pixels())
            .map(|(ours, theirs)| {
                // compute the pixel difference score
                let component_pairs = ours.0.as_ref().iter().zip(theirs.0);
                component_pairs
                    .map(|(a, b)| (a.abs_diff(b) as f32).powi(2))
                    .sum::<f32>()
            })
            .sum::<f32>();
        return image_score * 0.005;
    }
}
