pub use crate::moves::MoveType;
use derive_more::{Add, AddAssign, Sub, SubAssign, Sum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Add, AddAssign, Sub, SubAssign, Sum)]
pub struct Cost(pub u64);

impl Cost {
    pub fn base_cost(move_type: MoveType) -> f64 {
        match move_type {
            MoveType::LineCut => 7.0,
            MoveType::PointCut => 10.0,
            MoveType::Color => 5.0,
            MoveType::Swap => 3.0,
            MoveType::Merge => 1.0,
        }
    }

    pub fn compute(mov: MoveType, block_area: u32, canvas_area: u32) -> Cost {
        Cost((Cost::base_cost(mov) * (canvas_area as f64 / block_area as f64)).round() as u64)
    }
}
