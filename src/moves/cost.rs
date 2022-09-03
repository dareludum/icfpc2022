pub use crate::moves::Move;
use derive_more::{Add, AddAssign, Sub, Sum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Add, AddAssign, Sub, Sum)]
pub struct Cost(pub u64);

fn base_cost(mov: &Move) -> u32 {
    match mov {
        Move::LineCut(..) => 7,
        Move::PointCut(..) => 10,
        Move::Color(..) => 5,
        Move::Swap(..) => 3,
        Move::Merge(..) => 1,
    }
}

impl Cost {
    pub fn compute(mov: &Move, block_area: u32, canvas_area: u32) -> Cost {
        Cost((base_cost(mov) as f64 * (canvas_area as f64 / block_area as f64)).round() as u64)
    }
}
