use std::{collections::HashMap, path::Path};

use image::{Rgba, RgbaImage};

use crate::{
    block::{Block, BlockId, ComplexBlock, Point, Rect, SimpleBlock},
    color::Color,
    dto::CanvasDto,
    painting::Painting,
};

#[derive(Debug, Clone, PartialEq, Eq)]
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
        let blocks = [SimpleBlock::new(
            BlockId::from("0"),
            Rect::from_dimensions(Point::new(0, 0), w, h),
            Color::new(255, 255, 255, 255),
        )
        .into()];
        Self::from_blocks(w, h, 1, blocks.into_iter())
    }

    pub fn from_blocks(
        w: u32,
        h: u32,
        roots_count: u32,
        blocks: impl Iterator<Item = Block>,
    ) -> Self {
        let mut blocks_map = HashMap::new();
        for block in blocks {
            blocks_map.insert(block.get_id().clone(), block);
        }
        Canvas {
            width: w,
            height: h,
            area: w * h,
            blocks: blocks_map,
            roots_count,
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
                Block::Simple(simple_block) => render_simple_block(&mut img, simple_block),
                Block::Complex(ComplexBlock { bs: blocks, .. }) => {
                    blocks.iter().for_each(|b| render_simple_block(&mut img, b))
                }
            }
        }

        Painting::from_image(img)
    }

    // TODO impl from trait instead
    pub fn load_canvas(path: &Path) -> std::io::Result<Self> {
        let txt = std::fs::read_to_string(path)?;
        let dto: CanvasDto = serde_json::from_str(&txt)?;
        let mut canvas = Canvas::new(dto.width, dto.height);
        dto.blocks
            .iter()
            .for_each(|bdto| canvas.put_block(bdto.into()));

        Ok(canvas)
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
            img.put_pixel(x, img.height() - 1 - y, Rgba([*r, *g, *b, *a]))
        }
    }
}
