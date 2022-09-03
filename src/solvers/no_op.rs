use crate::{canvas::Canvas, moves::AppliedMove, painting::Painting};

use super::Solver;

pub struct NoOp {}

impl Solver for NoOp {
    fn name(&self) -> &'static str {
        "no_op"
    }

    fn solve_core(&self, _canvas: &mut Canvas, _painting: &Painting) -> Vec<AppliedMove> {
        vec![]
    }
}
