#[derive(Clone, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Rect { x, y, w, h }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

pub type BlockId = Vec<u32>;

pub enum Block {
    SimpleBlock(BlockId, Rect, Color),
    ComplexBlock(BlockId, Rect, Vec<Block>),
}

impl Block {
    pub fn id(&self) -> &BlockId {
        match self {
            Block::SimpleBlock(id, _, _) => id,
            Block::ComplexBlock(id, _, _) => id,
        }
    }

    pub fn rect(&self) -> &Rect {
        match self {
            Block::SimpleBlock(_, rect, _) => rect,
            Block::ComplexBlock(_, rect, _) => rect,
        }
    }
}
