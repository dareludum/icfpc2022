mod annealing;
mod chain;
mod divide_conquer;
mod erase;
mod no_op;
mod processors;
mod simple;
mod swapper;
mod top_color;

use crate::{
    canvas::Canvas,
    moves::{AppliedMove, Cost, Move},
    painting::Painting,
};

use self::chain::Chain;

pub struct Solution {
    pub result: Painting,
    pub moves: Vec<Move>,
    pub cost: Cost,
}

pub trait Solver {
    fn name(&self) -> &str;
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

pub trait Processor {
    fn name(&self) -> &str;
    fn process(
        &self,
        applied_moves: &mut Vec<AppliedMove>,
        canvas: &mut Canvas,
        painting: &Painting,
    );
}

pub const SOLVERS: &[&str] = &[
    "annealing",
    "annealing_s4",
    "avg_color",
    "divide_conquer",
    "erase",
    "no_op",
    "simple",
    "simple_no_x",
    "simple_s1",
    "simple_no_x_s1",
    "swapper",
    "top_color",
];

pub fn create_solver(input_moves: Option<Vec<Move>>, solver_name: &str) -> Box<dyn Solver> {
    if solver_name.contains(&['+', '%']) {
        let (solver_name, processor_name) = if solver_name.contains('%') {
            let parts = solver_name.split_at(solver_name.find('%').unwrap());
            (parts.0, Some(&parts.1[1..]))
        } else {
            (solver_name, None)
        };
        let mut solvers = vec![];
        for name in solver_name.split('+') {
            solvers.push(create_individual_solver(name))
        }
        let mut processors = vec![];
        if let Some(processor_name) = processor_name {
            for name in processor_name.split('+') {
                processors.push(create_processor(name))
            }
        }
        Box::new(Chain::new(input_moves, solvers, processors))
    } else {
        create_individual_solver(solver_name)
    }
}

fn create_individual_solver(solver_name: &str) -> Box<dyn Solver> {
    match solver_name {
        "annealing" => Box::new(annealing::Annealing { step: 10 }),
        "annealing_s4" => Box::new(annealing::Annealing { step: 4 }),
        "avg_color" => Box::new(top_color::TopColor { use_avg: true }),
        "divide_conquer" => Box::new(divide_conquer::DivideConquerSolver {}),
        "erase" => Box::new(erase::Erase {}),
        "no_op" => Box::new(no_op::NoOp {}),
        "simple" => Box::new(simple::Simple {
            allow_cross_cut: true,
            step_1: false,
        }),
        "simple_no_x" => Box::new(simple::Simple {
            allow_cross_cut: false,
            step_1: false,
        }),
        "simple_s1" => Box::new(simple::Simple {
            allow_cross_cut: true,
            step_1: true,
        }),
        "simple_no_x_s1" => Box::new(simple::Simple {
            allow_cross_cut: false,
            step_1: true,
        }),
        "swapper" => Box::new(swapper::Swapper {}),
        "top_color" => Box::new(top_color::TopColor { use_avg: false }),
        n => panic!("Unknown solver `{}`", n),
    }
}

fn create_processor(processor_name: &str) -> Box<dyn Processor> {
    match processor_name {
        "recolor" => Box::new(processors::recolor::Recolor {}),
        "shake" => Box::new(processors::shake::Shake {}),
        n => panic!("Unknown procesor `{}`", n),
    }
}
