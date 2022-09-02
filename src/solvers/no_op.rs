use crate::painting::Painting;

use super::Solver;

pub struct NoOp {}

impl Solver for NoOp {
    fn name(&self) -> &'static str {
        "no_op"
    }

    fn solve(&self, _painting: &Painting) -> Vec<crate::moves::Move> {
        vec![]
    }
}
