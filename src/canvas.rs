use std::collections::HashMap;

use crate::block::{Block, BlockId, Color, Rect, SimpleBlock};

pub struct Canvas {
    size: u32,
    blocks: HashMap<BlockId, Block>,
}

impl Canvas {
    pub fn new(w: u32, h: u32) -> Self {
        let mut blocks = HashMap::new();
        blocks.insert(
            BlockId::from("0"),
            Block::Simple(SimpleBlock::new(
                BlockId::from("0"),
                Rect::new(0, 0, w, h),
                Color::new(255, 255, 255, 255),
            )),
        );
        Canvas {
            size: w * h,
            blocks,
        }
    }

    // TODO: change to iterate instead of creating a vec
    pub fn blocks_iter(&self) -> Vec<&Block> {
        let mut blocks = vec![];
        // for b in self.blocks {
        // match b {
        //     Block::Simple(_, _, _) => {
        //         blocks.push()
        //     },
        //     Block::ComplexBlock(_, _, _) => todo!(),
        // }
        // }
        blocks
    }
}
