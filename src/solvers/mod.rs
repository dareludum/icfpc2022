mod divide_conquer;
mod no_op;

use crate::{
    canvas::Canvas,
    moves::{Cost, Move},
    painting::Painting,
};

pub struct Solution {
    pub result: Painting,
    pub moves: Vec<Move>,
    pub cost: Cost,
}

pub trait Solver {
    fn name(&self) -> &'static str;
    fn solve_core(&self, canvas: &mut Canvas, painting: &Painting) -> (Vec<Move>, Cost);

    fn solve(&self, painting: &Painting) -> Solution {
        let mut canvas = Canvas::new(painting.width(), painting.height());
        let (moves, cost) = self.solve_core(&mut canvas, painting);
        Solution {
            result: canvas.render(),
            moves,
            cost,
        }
    }
}

pub const SOLVERS: &[&'static str] = &["no_op", "divide_conquer"];

pub fn create_solver(solver_name: &str) -> Box<dyn Solver> {
    match solver_name {
        "no_op" => Box::new(no_op::NoOp {}),
        "divide_conquer" => Box::new(divide_conquer::DivideConquerSolver {}),
        n => panic!("Unknown solver `{}`", n),
    }
}
