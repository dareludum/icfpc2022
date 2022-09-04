use crate::{canvas::Canvas, moves::AppliedMove, painting::Painting};

use super::Solver;

pub struct Chain {
    name: String,
    solvers: Vec<Box<dyn Solver>>,
}

impl Solver for Chain {
    fn name(&self) -> &str {
        &self.name
    }

    fn solve_core(&self, canvas: &mut Canvas, painting: &Painting) -> Vec<AppliedMove> {
        let mut applied_moves = vec![];
        for s in &self.solvers {
            applied_moves.extend(s.solve_core(canvas, painting));
        }
        applied_moves
    }
}

impl Chain {
    pub fn new(solvers: Vec<Box<dyn Solver>>) -> Self {
        let mut name = String::new();
        for s in &solvers {
            name.push_str(s.name());
            name.push('+');
        }
        name = name.trim_end_matches('+').to_owned();
        Chain { name, solvers }
    }
}
