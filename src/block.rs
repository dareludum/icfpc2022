use image::Rgba;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rect {
    pub bottom_left: Point,
    pub top_right: Point,
}

impl Rect {
    pub const fn new(bottom_left: Point, top_right: Point) -> Self {
        Rect {
            bottom_left,
            top_right,
        }
    }

    pub const fn from_dimensions(bottom_left: Point, width: u32, height: u32) -> Self {
        Rect::new(
            bottom_left,
            Point::new(bottom_left.x + width - 1, bottom_left.y + height - 1),
        )
    }

    pub fn x(&self) -> u32 {
        self.bottom_left.x
    }

    pub fn y(&self) -> u32 {
        self.bottom_left.y
    }

    pub fn width(&self) -> u32 {
        self.top_right.x - self.bottom_left.x + 1
    }

    pub fn height(&self) -> u32 {
        self.top_right.y - self.bottom_left.y + 1
    }

    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.bottom_left.x
            && x <= self.top_right.x
            && y >= self.bottom_left.y
            && y <= self.top_right.y
    }

    pub fn vertical_cut(&self, x: u32) -> (Self, Self) {
        let left = Rect::new(self.bottom_left, Point::new(x, self.top_right.y));
        let right = Rect::new(Point::new(x, self.bottom_left.y), self.top_right);
        (left, right)
    }

    pub fn horizontal_cut(&self, y: u32) -> (Self, Self) {
        let bottom = Rect::new(self.bottom_left, Point::new(self.top_right.x, y));
        let top = Rect::new(Point::new(self.bottom_left.x, y), self.top_right);
        (bottom, top)
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

pub struct SimpleBlock {
    pub id: BlockId,
    pub r: Rect,
    pub c: Color,
}

impl SimpleBlock {
    pub fn new(id: BlockId, r: Rect, c: Color) -> Self {
        SimpleBlock { id, r, c }
    }

    pub fn split(&self, i: u32, r: Rect) -> Self {
        Self::new(format!("{}.{}", self.id, i), r, self.c)
    }

    /// Called when splitting a complex block
    pub fn complex_split(&self, r: Rect) -> Self {
        Self::new("child".to_owned(), r, self.c)
    }
}

pub struct ComplexBlock {
    pub id: BlockId,
    pub r: Rect,
    pub bs: Vec<SimpleBlock>,
}

impl ComplexBlock {
    pub fn new(id: BlockId, r: Rect, bs: Vec<SimpleBlock>) -> Self {
        ComplexBlock { id, r, bs }
    }
}

pub enum Block {
    Simple(SimpleBlock),
    Complex(ComplexBlock),
}

impl Into<Block> for SimpleBlock {
    fn into(self) -> Block {
        Block::Simple(self)
    }
}

impl Into<Block> for ComplexBlock {
    fn into(self) -> Block {
        Block::Complex(self)
    }
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
        self.rect().width() * self.rect().height()
    }
}
