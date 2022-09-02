use std::collections::HashMap;

use crate::block::{Block, BlockId, Color, Point, Rect, SimpleBlock};

#[derive(Debug)]
pub struct Canvas {
    pub area: u32,
    blocks: HashMap<BlockId, Block>,
}

impl Canvas {
    pub fn new(w: u32, h: u32) -> Self {
        let mut blocks = HashMap::new();
        blocks.insert(
            BlockId::from("0"),
            Block::Simple(SimpleBlock::new(
                BlockId::from("0"),
                Rect::from_dimensions(Point::new(0, 0), w, h),
                Color::new(255, 255, 255, 255),
            )),
        );
        Canvas {
            area: w * h,
            blocks,
        }
    }

    // TODO: change to iterate instead of creating a vec
    pub fn blocks_iter(&self) -> Vec<&SimpleBlock> {
        let mut blocks = vec![];
        for b in self.blocks.values() {
            match b {
                Block::Simple(b) => {
                    blocks.push(b);
                }
                Block::Complex(b) => {
                    for b in b.bs.iter() {
                        blocks.push(b);
                    }
                }
            }
        }
        blocks
    }

    pub fn hit_test(&self, x: u32, y: u32) -> String {
        for b in self.blocks.values() {
            if b.rect().contains(x, y) {
                return b.id().clone();
            }
        }
        panic!("Programmer error: hit test didn't find any blocks for ({}, {})", x, y);
    }

    pub fn get_block(&self, block: &BlockId) -> Option<&Block> {
        self.blocks.get(block)
    }

    pub fn put_block(&mut self, block: Block) {
        self.blocks.insert(block.id().clone(), block);
    }

    pub fn get_block_mut(&mut self, block: &BlockId) -> Option<&mut Block> {
        self.blocks.get_mut(block)
    }

    pub fn remove_block(&mut self, block: &BlockId) -> Option<Block> {
        self.blocks.remove(block)
    }
}
