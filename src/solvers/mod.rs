mod annealing;
mod chain;
mod divide_conquer;
mod erase;
mod no_op;
mod processors;
mod simple;
mod swapper;
mod top_color;

use std::path::PathBuf;

use dyn_clone::DynClone;

use crate::{
    canvas::Canvas,
    dto::SolvedSolutionDto,
    helpers::os_str_to_str,
    moves::{AppliedMove, Cost, Move},
    painting::Painting,
    program,
};

use self::chain::Chain;

pub struct Problem {
    pub id: String,
    pub reference_painting: Painting,
    pub initial_canvas: Canvas,
}

impl Problem {
    pub fn load(problem_path: &PathBuf) -> std::io::Result<Self> {
        let id = os_str_to_str(problem_path.file_stem());
        let reference_painting = Painting::load(problem_path);
        let initial_canvas =
            Canvas::try_create(problem_path.with_extension("json"), &reference_painting)?;
        Ok(Problem {
            id,
            reference_painting,
            initial_canvas,
        })
    }
}

pub struct Solution {
    pub result: Painting,
    pub moves: Vec<Move>,
    pub cost: Cost,
}

impl Solution {
    pub fn load(dir: &PathBuf, problem: &Problem) -> std::io::Result<(Self, SolvedSolutionDto)> {
        let problem_base = dir.join(&problem.id);
        let isl_path = problem_base.with_extension("txt");
        let img_path = problem_base.with_extension("png");
        let meta_path = problem_base.with_file_name(format!("{}_meta.json", problem.id));

        let moves = crate::parser::parse_moves_from_file(&isl_path)?;
        let result = Painting::load(&img_path);

        let current_best_json: String = std::fs::read_to_string(meta_path)?.into();
        let metadata: SolvedSolutionDto =
            serde_json::from_str(&current_best_json).expect("Deserialization error");
        let solution = Solution {
            result,
            moves,
            cost: Cost(metadata.solution_cost),
        };
        Ok((solution, metadata))
    }

    pub fn save(
        &self,
        solver_name: String,
        problem: &Problem,
        dir: &PathBuf,
    ) -> std::io::Result<SolvedSolutionDto> {
        let problem_base = dir.join(&problem.id);
        let isl_path = problem_base.with_extension("txt");
        let img_path = problem_base.with_extension("png");
        let meta_path = problem_base.with_file_name(format!("{}_meta.json", problem.id));

        program::write_to_file(&isl_path, &self.moves)?;
        self.result.write_to_file(&img_path);

        let score = problem.reference_painting.calculate_score(&self.result);
        let total = score + self.cost;
        let solution_meta = SolvedSolutionDto {
            solver_name: solver_name,
            score: score.0,
            total_score: total.0,
            solution_cost: self.cost.0,
        };
        let solution_meta_json = serde_json::to_string_pretty(&solution_meta)?;
        std::fs::write(meta_path, solution_meta_json)?;
        Ok(solution_meta)
    }
}

pub trait Solver: DynClone + Sync + Send {
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

dyn_clone::clone_trait_object!(Solver);

pub trait Processor: DynClone + Sync + Send {
    fn name(&self) -> &str;
    fn process(
        &self,
        applied_moves: &mut Vec<AppliedMove>,
        canvas: &mut Canvas,
        painting: &Painting,
    );
}

dyn_clone::clone_trait_object!(Processor);

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
        if !solver_name.is_empty() {
            for name in solver_name.split('+') {
                solvers.push(create_individual_solver(name))
            }
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
        "recolorv2" => Box::new(processors::recolorv2::Recolorv2 {}),
        "shake" => Box::new(processors::shake::Shake {}),
        n => panic!("Unknown procesor `{}`", n),
    }
}
