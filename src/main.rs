extern crate derive_more;

use std::path::{self, Path, PathBuf};

use clap::Parser;

use gui::gui_main;
use painting::Painting;
use solvers::{create_solver, SOLVERS};

mod block;
mod canvas;
mod color;
mod gui;
mod moves;
mod painting;
mod program;
mod solvers;

#[derive(Parser, Debug)]
#[clap()]
struct Args {
    #[clap(long)]
    batch: bool,
    #[clap(short, long, value_parser)]
    problem: Option<String>,
    #[clap(short, long)]
    solver: Option<String>,
}

fn solve(solvers: &[String], problem_paths: &[&Path]) -> std::io::Result<()> {
    for problem_path in problem_paths {
        let mut solution_filename = problem_path.file_stem().unwrap().to_owned();
        solution_filename.push(".txt");
        let solution_painting_filename = problem_path.file_name().unwrap().to_owned();

        println!("Processing {:?}", solution_painting_filename);
        let painting = Painting::load(&problem_path);

        for solver_name in solvers {
            let solver = create_solver(solver_name);

            let mut solution_dir = std::path::PathBuf::from("./solutions/");
            solution_dir.push(solver.name());
            std::fs::create_dir_all(&solution_dir)?;

            let solution = solver.solve(&painting);
            program::write_to_file(&solution_dir.join(&solution_filename), &solution.moves)?;
            solution
                .result
                .write_to_file(&solution_dir.join(&solution_painting_filename));

            let score = painting.calculate_score(&solution.result);
            let total = score + solution.cost;
            println!(
                "  {}: {} ({} + {})",
                solver.name(),
                total.0,
                score.0,
                solution.cost.0
            );
        }
    }

    Ok(())
}

fn get_problem_paths(args: &Args) -> Result<Vec<PathBuf>, std::io::Error> {
    if let Some(problem) = args.problem.clone() {
        Ok(vec![PathBuf::from(&problem)])
    } else if args.batch {
        let paths: Result<Vec<PathBuf>, _> = std::fs::read_dir("./problems")?
            .map(|f_res| f_res.map(|f| f.path()))
            .collect();
        paths
    } else {
        Ok(vec![PathBuf::from("./problems/3.png")])
    }
}

fn get_solvers(args: &Args) -> Option<Vec<String>> {
    if let Some(solver) = args.solver.clone() {
        Some(vec![solver])
    } else if args.batch {
        Some(SOLVERS.iter().map(|s| s.to_string()).collect())
    } else {
        None
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let problem_paths_buf = get_problem_paths(&args)?;
    let problem_paths: Vec<&Path> = problem_paths_buf
        .iter()
        .map(|path| path.as_path())
        .collect();

    let solvers = get_solvers(&args);

    match (&problem_paths[..], solvers) {
        ([problem_path], None) => {
            gui_main(&std::path::PathBuf::from(problem_path));
            Ok(())
        }
        (paths, Some(solvers)) => solve(&solvers, &paths),
        (_, None) => panic!("No problem paths and solvers provided"),
    }
}
