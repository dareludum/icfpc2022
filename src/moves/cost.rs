pub use crate::moves::MoveType;
use derive_more::{Add, AddAssign, Sub, SubAssign, Sum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Add, AddAssign, Sub, SubAssign, Sum)]
pub struct Cost(pub u64);
