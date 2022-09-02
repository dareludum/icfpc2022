mod no_op;

use crate::{moves::Move, painting::Painting};

pub struct Solution {
    pub result: Painting,
    pub moves: Vec<Move>,
    pub cost: u32,
}

pub trait Solver {
    fn name(&self) -> &'static str;
    fn solve(&self, painting: &Painting) -> Solution;
}

pub const SOLVERS: &[&'static str] = &["no_op"];

pub fn create_solver(solver_name: &str) -> Box<dyn Solver> {
    match solver_name {
        "no_op" => Box::new(no_op::NoOp {}),
        n => panic!("Unknown solver `{}`", n),
    }
}
