use crate::block::{BlockId, Color};

pub enum Orientation {
    Horizontal,
    Vertical,
}

pub enum Move {
    LineCut(BlockId, Orientation, u32),
    PointCut(BlockId, u32, u32),
    Color(BlockId, Color),
    Swap(BlockId, BlockId),
    Merge(BlockId, BlockId),
}

impl Move {
    pub fn cost(&self) -> u32 {
        match self {
            Move::LineCut(_, _, _) => 7,
            Move::PointCut(_, _, _) => 10,
            Move::Color(_, _) => 5,
            Move::Swap(_, _) => 3,
            Move::Merge(_, _) => 1,
        }
    }
}
