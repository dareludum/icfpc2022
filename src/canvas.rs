use crate::block::Block;

pub struct Canvas {
    size: u32,
    blocks: Vec<Block>,
}

impl Canvas {
    pub fn new(w: u32, h: u32) -> Self {
        Canvas { size: w * h, blocks: vec![] }
    }
}
