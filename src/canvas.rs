use crate::block::{Block, Color, Rect};

pub struct Canvas {
    size: u32,
    blocks: Vec<Block>,
}

impl Canvas {
    pub fn new(w: u32, h: u32) -> Self {
        Canvas {
            size: w * h,
            blocks: vec![Block::SimpleBlock(
                vec![0],
                Rect::new(0, 0, w, h),
                Color::new(255, 255, 255, 255),
            )],
        }
    }
}
