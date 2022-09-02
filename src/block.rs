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
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }
}

impl Into<Rgba<u8>> for &Color {
    fn into(self) -> Rgba<u8> {
        Rgba([self.r, self.g, self.b, self.a])
    }
}

impl From<&Rgba<u8>> for Color {
    fn from(src: &Rgba<u8>) -> Self {
        Color {
            r: src[0],
            g: src[1],
            b: src[2],
            a: src[3],
        }
    }
}

pub type BlockId = String;

pub struct SimpleBlock { id: BlockId, r: Rect, c: Color }

impl SimpleBlock {
    pub fn new(id: BlockId, r: Rect, c: Color) -> Self {
        SimpleBlock { id, r, c }
    }
}

pub struct ComplexBlock { id: BlockId, r: Rect, bs: Vec<SimpleBlock> }

impl ComplexBlock {
    pub fn new(id: BlockId, r: Rect, bs: Vec<SimpleBlock>) -> Self {
        ComplexBlock { id, r, bs }
    }
}

pub enum Block {
    Simple(SimpleBlock),
    Complex(ComplexBlock),
}

impl Block {
    pub fn id(&self) -> &BlockId {
        match self {
            Block::Simple(b) => &b.id,
            Block::Complex(b) => &b.id,
        }
    }

    pub fn rect(&self) -> &Rect {
        match self {
            Block::Simple(b) => &b.r,
            Block::Complex(b) => &b.r,
        }
    }

    pub fn size(&self) -> u32 {
        self.rect().w * self.rect().h
    }
}
