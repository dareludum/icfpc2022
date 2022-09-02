use crate::{canvas::Canvas, painting::Painting};

use super::{Solution, Solver};

pub struct NoOp {}

impl Solver for NoOp {
    fn name(&self) -> &'static str {
        "no_op"
    }

    fn solve(&self, painting: &Painting) -> Solution {
        Solution {
            result: Canvas::new(painting.width(), painting.height()).render(),
            moves: vec![],
            cost: 0,
        }
    }
}
