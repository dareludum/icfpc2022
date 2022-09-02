use image::Rgba;

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

impl Into<Rgba<u8>> for &Color {
    fn into(self) -> Rgba<u8> {
        Rgba([self.r, self.g, self.b, self.a])
    }
}

impl From<&Rgba<u8>> for Color {
    fn from(src: &Rgba<u8>) -> Self {
        Color{ r: src[0], g: src[1], b: src[2], a: src[3] }
    }
}

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

    pub fn size(&self) -> u32 {
        self.rect().w * self.rect().h
    }
}
