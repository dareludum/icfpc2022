use crate::{color::Color, dto::BlockDto};

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
    /// inclusive lower bound
    pub bottom_left: Point,
    /// exclusive upper bound
    pub top_right: Point,
}

impl Rect {
    pub const fn from_coords(coords: [u32; 4]) -> Self {
        Rect::new(
            Point::new(coords[0], coords[1]),
            Point::new(coords[2], coords[3]),
        )
    }

    pub const fn new(bottom_left: Point, top_right: Point) -> Self {
        Rect {
            bottom_left,
            top_right,
        }
    }

    pub const fn from_dimensions(bottom_left: Point, width: u32, height: u32) -> Self {
        Rect::new(
            bottom_left,
            Point::new(bottom_left.x + width, bottom_left.y + height),
        )
    }

    pub fn x(&self) -> u32 {
        self.bottom_left.x
    }

    pub fn y(&self) -> u32 {
        self.bottom_left.y
    }

    pub fn center(&self) -> Point {
        Point {
            x: self.x() + self.width() / 2,
            y: self.y() + self.height() / 2,
        }
    }

    pub fn width(&self) -> u32 {
        self.top_right.x - self.bottom_left.x
    }

    pub fn height(&self) -> u32 {
        self.top_right.y - self.bottom_left.y
    }

    pub fn area(&self) -> u32 {
        self.width() * self.height()
    }

    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.bottom_left.x
            && x < self.top_right.x
            && y >= self.bottom_left.y
            && y < self.top_right.y
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

    pub fn cross_cut(&self, x: u32, y: u32) -> (Self, Self, Self, Self) {
        let cut_point = Point::new(x, y);
        let bottom_left = &self.bottom_left;
        let top_right = &self.top_right;
        let bottom_left_bl = Rect::new(*bottom_left, cut_point);
        let bottom_right_bl = Rect::new(Point::new(x, bottom_left.y), Point::new(top_right.x, y));
        let top_right_bl = Rect::new(cut_point, *top_right);
        let top_left_bl = Rect::new(Point::new(bottom_left.x, y), Point::new(x, top_right.y));
        (bottom_left_bl, bottom_right_bl, top_right_bl, top_left_bl)
    }
}

pub type BlockId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn complex_split(&self, name: &'static str, r: Rect) -> Self {
        Self::new(name.to_owned(), r, self.c)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Simple(SimpleBlock),
    Complex(ComplexBlock),
}

impl From<SimpleBlock> for Block {
    fn from(b: SimpleBlock) -> Self {
        Block::Simple(b)
    }
}

impl From<ComplexBlock> for Block {
    fn from(b: ComplexBlock) -> Self {
        Block::Complex(b)
    }
}

impl From<&BlockDto> for Block {
    fn from(dto: &BlockDto) -> Self {
        let BlockDto {
            block_id,
            bottom_left: [bl_x, bl_y],
            top_right: [tr_x, tr_y],
            color: [r, g, b, a],
        } = dto;

        Block::Simple(SimpleBlock::new(
            block_id.clone(),
            Rect::from_coords([*bl_x, *bl_y, *tr_x, *tr_y]),
            Color::new(*r, *g, *b, *a),
        ))
    }
}

impl Block {
    pub fn get_id(&self) -> &BlockId {
        match self {
            Block::Simple(b) => &b.id,
            Block::Complex(b) => &b.id,
        }
    }

    pub fn get_id_mut(&mut self) -> &mut BlockId {
        match self {
            Block::Simple(b) => &mut b.id,
            Block::Complex(b) => &mut b.id,
        }
    }

    pub fn rect(&self) -> &Rect {
        match self {
            Block::Simple(b) => &b.r,
            Block::Complex(b) => &b.r,
        }
    }

    pub fn take_children(self) -> Vec<SimpleBlock> {
        match self {
            Block::Simple(b) => vec![b],
            Block::Complex(b) => b.bs,
        }
    }

    pub fn size(&self) -> u32 {
        self.rect().width() * self.rect().height()
    }
}
