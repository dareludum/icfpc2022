use crate::block::{Block, BlockData, BlockId, Rect};
use crate::canvas::Canvas;
use crate::color::Color;
use crate::moves::Cost;
use image::{Rgba, RgbaImage};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

impl From<Color> for Rgba<u8> {
    fn from(c: Color) -> Self {
        Rgba([c.r(), c.g(), c.b(), c.a()])
    }
}

impl From<&Rgba<u8>> for Color {
    fn from(src: &Rgba<u8>) -> Self {
        Color::new(src[0], src[1], src[2], src[3])
    }
}

#[derive(Debug)]
pub struct Painting {
    pub width: u32,
    pub height: u32,
    data: Vec<Color>,
}

impl Painting {
    pub fn from_image(image: RgbaImage) -> Self {
        let mut data = vec![];
        let width = image.width();
        let height = image.height();
        data.resize((width * height) as usize, Color::new(0, 0, 0, 0));
        for x in 0..width {
            for y in 0..image.height() {
                data[(x + width * y) as usize] = image.get_pixel(x, height - y - 1).into();
            }
        }
        Painting {
            width,
            height,
            data,
        }
    }

    pub fn new(width: u32, height: u32, data: Vec<Color>) -> Self {
        Painting {
            width,
            height,
            data,
        }
    }

    pub fn load<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Self {
        let file = match File::open(&path) {
            Err(why) => panic!("couldn't open {:?}: {}", path, why),
            Ok(file) => file,
        };
        let mut reader = BufReader::new(file);
        let dyn_img =
            image::load(&mut reader, image::ImageFormat::Png).expect("image loading failed");
        Painting::from_image(dyn_img.into_rgba8())
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn get_color(&self, x: u32, y: u32) -> Color {
        self.data[(x + y * self.width) as usize]
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

    pub fn get_pixels(&self, r: &Rect) -> Vec<Color> {
        let mut res = Vec::with_capacity(r.area() as usize);
        for x in r.x()..r.top_right.x {
            for y in r.y()..r.top_right.y {
                res.push(self.get_color(x, y));
            }
        }
        res
    }

    pub fn calculate_average_color(&self, rect: &Rect) -> Color {
        let total_pixels = rect.area() as u64;
        let mut r = 0u64;
        let mut g = 0u64;
        let mut b = 0u64;
        let mut a = 0u64;
        for x in rect.x()..rect.top_right.x {
            for y in rect.y()..rect.top_right.y {
                let c = self.get_color(x, y);
                r += c.r() as u64;
                g += c.g() as u64;
                b += c.b() as u64;
                a += c.a() as u64;
            }
        }
        Color::new(
            (r / total_pixels) as u8,
            (g / total_pixels) as u8,
            (b / total_pixels) as u8,
            (a / total_pixels) as u8,
        )
    }

    pub fn find_worst_block_id<'a>(&self, canvas: &'a Canvas) -> &'a BlockId {
        let mut worst_block = None;
        let mut worst_score = 0.0;
        for b in canvas.blocks_iter() {
            let score = self.calculate_score_block(b);
            if score > worst_score {
                worst_score = score;
                worst_block = Some(&b.id);
            }
        }
        worst_block.unwrap()
    }

    pub fn calculate_abs_diff_map(&self, target: &Canvas) -> Vec<f64> {
        let mut map = vec![];
        map.resize(self.data.len(), 0.0);
        for b in target.blocks_iter() {
            match &b.data {
                BlockData::Simple(c) => {
                    for x in b.r.x()..b.r.top_right.x {
                        for y in b.r.y()..b.r.top_right.y {
                            map[(x + y * self.width) as usize] =
                                self.calculate_score_pixel(x, y, *c);
                        }
                    }
                }
                BlockData::Complex(bs) => {
                    for b in bs {
                        for x in b.r.x()..b.r.top_right.x {
                            for y in b.r.y()..b.r.top_right.y {
                                map[(x + y * self.width) as usize] =
                                    self.calculate_score_pixel(x, y, b.c);
                            }
                        }
                    }
                }
            }
        }
        map
    }

    pub fn calculate_score(&self, target: &Painting) -> Cost {
        if target.width() != self.width() || target.height() != self.height() {
            panic!("comparing two images different in size");
        }

        // compute the image difference score
        let image_score = self
            .data
            .iter()
            .zip(target.data.iter())
            .map(|(c0, c1)| {
                let mut pixel_score = 0f64;
                pixel_score += (c0.r().abs_diff(c1.r()) as f64).powi(2);
                pixel_score += (c0.g().abs_diff(c1.g()) as f64).powi(2);
                pixel_score += (c0.b().abs_diff(c1.b()) as f64).powi(2);
                pixel_score += (c0.a().abs_diff(c1.a()) as f64).powi(2);
                pixel_score.sqrt()
            })
            .sum::<f64>();
        Cost::from_block_cost(image_score)
    }

    pub fn calculate_score_canvas(&self, target: &Canvas) -> Cost {
        if target.width != self.width() || target.height != self.height() {
            panic!("comparing two images different in size");
        }

        let mut image_score = 0.0;
        for b in target.blocks_iter() {
            image_score += self.calculate_score_block(b);
        }
        Cost::from_block_cost(image_score)
    }

    pub fn calculate_score_block(&self, b: &Block) -> f64 {
        let mut block_score = 0.0;
        match &b.data {
            BlockData::Simple(c) => {
                block_score += self.calculate_score_rect(&b.r, *c);
            }
            BlockData::Complex(bs) => {
                for b in bs {
                    block_score += self.calculate_score_rect(&b.r, b.c);
                }
            }
        }
        block_score
    }

    pub fn calculate_score_rect(&self, r: &Rect, c: Color) -> f64 {
        let mut block_score = 0.0;
        for x in r.x()..r.top_right.x {
            for y in r.y()..r.top_right.y {
                block_score += self.calculate_score_pixel(x, y, c);
            }
        }
        block_score
    }

    #[inline(always)]
    fn calculate_score_pixel(&self, x: u32, y: u32, c: Color) -> f64 {
        let pc = self.get_color(x, y);
        let mut pixel_score = 0f64;
        pixel_score += (c.r().abs_diff(pc.r()) as f64).powi(2);
        pixel_score += (c.g().abs_diff(pc.g()) as f64).powi(2);
        pixel_score += (c.b().abs_diff(pc.b()) as f64).powi(2);
        pixel_score += (c.a().abs_diff(pc.a()) as f64).powi(2);
        pixel_score.sqrt()
    }

    pub fn write_to_file(&self, path: &std::path::Path) {
        let mut img = RgbaImage::new(self.width, self.height);
        for x in 0..self.width {
            for y in 0..self.height {
                img.put_pixel(x, self.height - y - 1, self.get_color(x, y).into());
            }
        }

        img.save_with_format(path, image::ImageFormat::Png).unwrap();
    }
}
