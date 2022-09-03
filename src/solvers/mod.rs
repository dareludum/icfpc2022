mod annealing;
mod divide_conquer;
mod no_op;
mod top_color;

use crate::{
    canvas::Canvas,
    moves::{AppliedMove, Cost, Move},
    painting::Painting,
};

pub struct Solution {
    pub result: Painting,
    pub moves: Vec<Move>,
    pub cost: Cost,
}

pub trait Solver {
    fn name(&self) -> &'static str;
    fn solve_core(&self, canvas: &mut Canvas, painting: &Painting) -> Vec<AppliedMove>;

    fn solve(&self, canvas: &mut Canvas, painting: &Painting) -> Solution {
        let applied_moves = self.solve_core(canvas, painting);
        let mut cost = Cost(0);
        let mut moves = vec![];
        for am in applied_moves {
            cost += am.cost;
            moves.push(am.mov);
        }
        Solution {
            result: canvas.render(),
            moves,
            cost,
        }
    }
}

pub const SOLVERS: &[&str] = &["annealing", "no_op", "divide_conquer", "top_color"];

pub fn create_solver(solver_name: &str) -> Box<dyn Solver> {
    match solver_name {
        "annealing" => Box::new(annealing::Annealing {}),
        "no_op" => Box::new(no_op::NoOp {}),
        "divide_conquer" => Box::new(divide_conquer::DivideConquerSolver {}),
        "top_color" => Box::new(top_color::TopColor {}),
        n => panic!("Unknown solver `{}`", n),
    }
}
