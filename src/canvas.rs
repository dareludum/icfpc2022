use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    block::{Block, BlockData, BlockId, Point, Rect},
    color::Color,
    dto::CanvasDto,
    moves::{Cost, MoveType},
    painting::Painting,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Canvas {
    pub v2: bool, // Whether to use new costs or not
    area: u32,
    pub width: u32,
    pub height: u32,
    blocks: HashMap<BlockId, Block>,
    roots_count: u32,
    pub generation: u32,
}

impl From<CanvasDto> for Canvas {
    fn from(dto: CanvasDto) -> Self {
        let blocks: Vec<Block> = dto.blocks.iter().map(|bdto| bdto.into()).collect();

        let max_root: Option<u32> = blocks
            .iter()
            .filter_map(|block| {
                let id = &block.id.0;
                let root_id_str = match id.find('.') {
                    Some(dot_off) => &id.as_str()[0..dot_off],
                    None => id.as_str(),
                };
                // try to parse identifiers as integers, exclude from the max if parsing fails
                lexical_core::parse(root_id_str.as_bytes()).ok()
            })
            .max();

        let initial_root_count = max_root.map_or(0, |max_root| max_root + 1);

        Canvas::from_blocks(
            dto.width,
            dto.height,
            initial_root_count,
            0,
            blocks.into_iter(),
        )
    }
}

impl Canvas {
    pub fn next_merge_id(&mut self) -> BlockId {
        let res = self.roots_count;
        self.roots_count += 1;
        BlockId::new_root(res)
    }

    pub fn prev_merge_id(&mut self) {
        self.roots_count -= 1
    }

    pub fn next_generation(&mut self) -> u32 {
        self.generation += 1;
        self.generation
    }

    pub fn prev_generation(&mut self) {
        self.generation -= 1
    }

    pub fn new(w: u32, h: u32) -> Self {
        let blocks = [Block::new_simple(
            BlockId::from("0"),
            Rect::from_dimensions(Point::new(0, 0), w, h),
            Color::new(255, 255, 255, 255),
        )];
        Self::from_blocks(w, h, 1, 0, blocks.into_iter())
    }

    pub fn from_blocks(
        w: u32,
        h: u32,
        roots_count: u32,
        generation: u32,
        blocks: impl Iterator<Item = Block>,
    ) -> Self {
        let mut blocks_map = HashMap::new();
        for block in blocks {
            blocks_map.insert(block.id.clone(), block);
        }
        Canvas {
            v2: false,
            width: w,
            height: h,
            area: w * h,
            blocks: blocks_map,
            roots_count,
            generation,
        }
    }

    pub fn blocks_iter(&self) -> impl Iterator<Item = &Block> {
        self.blocks.values()
    }

    pub fn blocks_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn hit_test(&self, x: u32, y: u32) -> BlockId {
        for b in self.blocks.values() {
            if b.r.contains(x, y) {
                return b.id.clone();
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
        self.blocks.insert(block.id.clone(), block);
    }

    pub fn get_block_mut(&mut self, block: &BlockId) -> Option<&mut Block> {
        self.blocks.get_mut(block)
    }

    pub fn remove_block(&mut self, block: &BlockId) -> Option<Block> {
        self.blocks.remove(block)
    }

    fn base_cost(&self, move_type: MoveType) -> f64 {
        match move_type {
            MoveType::LineCut => {
                if self.v2 {
                    2.0
                } else {
                    7.0
                }
            }
            MoveType::PointCut => {
                if self.v2 {
                    3.0
                } else {
                    10.0
                }
            }
            MoveType::Color => 5.0,
            MoveType::Swap => 3.0,
            MoveType::Merge => 1.0,
        }
    }

    pub fn compute_cost(&self, mov: MoveType, block_area: u32) -> Cost {
        Cost((self.base_cost(mov) * (self.area as f64 / block_area as f64)).round() as u64)
    }

    pub fn render(&self) -> Painting {
        let mut data = vec![];
        data.resize((self.width * self.height) as usize, Color::new(0, 0, 0, 0));

        for (_, block) in self.blocks.iter() {
            match &block.data {
                BlockData::Simple(c) => self.render_rect(&mut data, &block.r, *c),
                BlockData::Complex(blocks) => blocks
                    .iter()
                    .for_each(|b| self.render_rect(&mut data, &b.r, b.c)),
            }
        }

        Painting::new(self.width, self.height, data)
    }

    fn render_rect(&self, data: &mut [Color], r: &Rect, c: Color) {
        let Rect {
            bottom_left,
            top_right,
        } = r;

        for x in bottom_left.x..top_right.x {
            for y in bottom_left.y..top_right.y {
                data[(x + self.width * y) as usize] = c;
            }
        }
    }

    fn load_canvas(path: &Path) -> std::io::Result<Self> {
        let txt = std::fs::read_to_string(path)?;
        let dto: CanvasDto = serde_json::from_str(&txt)?;
        Ok(dto.into())
    }

    pub fn try_create(
        initial_config_path: PathBuf,
        painting: &Painting,
    ) -> Result<Canvas, std::io::Error> {
        let canvas = match initial_config_path.try_exists() {
            Ok(true) => Canvas::load_canvas(&initial_config_path)?,
            Ok(false) => Canvas::new(painting.width(), painting.height()),
            Err(e) => return Err(e),
        };

        Ok(canvas)
    }
}
