pub use crate::moves::MoveType;
use derive_more::{Add, AddAssign, Sub, SubAssign, Sum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Add, AddAssign, Sub, SubAssign, Sum)]
pub struct Cost(pub u64);

impl Cost {
    pub fn from_block_cost(block_cost: f64) -> Cost {
        Cost((block_cost * 0.005).round() as u64)
    }
}
