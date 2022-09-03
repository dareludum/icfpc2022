use std::collections::HashMap;

use image::{Rgba, RgbaImage};

use crate::{
    block::{Block, BlockId, Color, ComplexBlock, Point, Rect, SimpleBlock},
    painting::Painting,
};

#[derive(Debug)]
pub struct Canvas {
    pub area: u32,
    width: u32,
    height: u32,
    blocks: HashMap<BlockId, Block>,
    roots_count: u32,
}

impl Canvas {
    pub fn next_merge_id(&mut self) -> String {
        let res = self.roots_count;
        self.roots_count += 1;
        res.to_string()
    }

    pub fn prev_merge_id(&mut self) {
        self.roots_count -= 1
    }

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
            width: w,
            height: h,
            area: w * h,
            blocks,
            roots_count: 1,
        }
    }

    // TODO: change to iterate instead of creating a vec
    pub fn blocks_iter(&self) -> Vec<&Block> {
        let mut blocks = vec![];
        for b in self.blocks.values() {
            blocks.push(b);
        }
        blocks
    }

    pub fn hit_test(&self, x: u32, y: u32) -> String {
        for b in self.blocks.values() {
            if b.rect().contains(x, y) {
                return b.get_id().clone();
            }
        }
        panic!(
            "Programmer error: hit test didn't find any blocks for ({}, {})",
            x, y
        );
    }

    pub fn get_block(&self, block: &BlockId) -> Option<&Block> {
        self.blocks.get(block)
    }

    pub fn put_block(&mut self, block: Block) {
        self.blocks.insert(block.get_id().clone(), block);
    }

    pub fn get_block_mut(&mut self, block: &BlockId) -> Option<&mut Block> {
        self.blocks.get_mut(block)
    }

    pub fn remove_block(&mut self, block: &BlockId) -> Option<Block> {
        self.blocks.remove(block)
    }

    pub fn render(&self) -> Painting {
        let mut img = RgbaImage::new(self.width, self.height);

        for (_, block) in self.blocks.iter() {
            match block {
                Block::Simple(simple_block) => render_simple_block(&mut img, &simple_block),
                Block::Complex(ComplexBlock { bs: blocks, .. }) => {
                    blocks.iter().for_each(|b| render_simple_block(&mut img, b))
                }
            }
        }

        Painting { image: img }
    }
}

fn render_simple_block(img: &mut RgbaImage, block: &SimpleBlock) {
    let SimpleBlock {
        r: Rect {
            bottom_left,
            top_right,
        },
        c: Color { r, g, b, a },
        ..
    } = block;

    for x in bottom_left.x..top_right.x {
        for y in bottom_left.y..top_right.y {
            img.put_pixel(x, y, Rgba([*r, *g, *b, *a]))
        }
    }
}
