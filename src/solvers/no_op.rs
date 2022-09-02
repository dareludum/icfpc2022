use crate::{
    canvas::Canvas,
    moves::{Cost, Move},
    painting::Painting,
};

use super::Solver;

pub struct NoOp {}

impl Solver for NoOp {
    fn name(&self) -> &'static str {
        "no_op"
    }

    fn solve_core(&self, _canvas: &mut Canvas, _painting: &Painting) -> (Vec<Move>, Cost) {
        (vec![], Cost(0))
    }
}
