use crate::{canvas::Canvas, moves::AppliedMove, painting::Painting};

use super::{Processor, Solver};

pub struct Chain {
    name: String,
    solvers: Vec<Box<dyn Solver>>,
    processors: Vec<Box<dyn Processor>>,
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
        for p in &self.processors {
            p.process(&mut applied_moves, canvas, painting);
        }
        applied_moves
    }
}

impl Chain {
    pub fn new(solvers: Vec<Box<dyn Solver>>, processors: Vec<Box<dyn Processor>>) -> Self {
        let mut name = String::new();
        for s in &solvers {
            name.push_str(s.name());
            name.push('+');
        }
        name = name.trim_end_matches('+').to_owned();
        if !processors.is_empty() {
            name.push('!');
            for p in &processors {
                name.push_str(p.name());
                name.push('+');
            }
            name = name.trim_end_matches('+').to_owned();
        }
        Chain {
            name,
            solvers,
            processors,
        }
    }
}
