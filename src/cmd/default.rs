use std::{
    fs,
    path::PathBuf,
};

use crate::{
    gui::gui_main,
    moves::Move,
    solvers::Problem,
    solvers::{create_solver, Solution, Solver},
};
use rayon::prelude::*;

fn solve_problem(
    solvers: &Vec<Box<dyn Solver>>,
    base_solution_dir: &PathBuf,
    problem_path: &PathBuf,
) -> std::io::Result<()> {
    let problem = Problem::load(problem_path)?;

    for solver in solvers {
        let full_solver_name = solver.name();
        let mut canvas = problem.initial_canvas.clone();
        let cur_solver_dir = &base_solution_dir.join("current").join(full_solver_name);
        let best_dir = &base_solution_dir.join("best");
        std::fs::create_dir_all(cur_solver_dir)?;

        // solve
        let solution = solver.solve(&mut canvas, &problem.reference_painting);

        // write the solution
        let solution_meta = solution.save(full_solver_name.into(), &problem, cur_solver_dir)?;

        // compare with the best solution
        let best_sol = match Solution::load(best_dir, &problem) {
            Ok(sol) => Some(sol),
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e),
        };

        let new_best_sol = match &best_sol {
            Some((_, best_sol)) if solution_meta.total_score < best_sol.total_score => true,
            None => true,
            _ => false,
        };

        if new_best_sol {
            solution.save(full_solver_name.into(), &problem, best_dir)?;
        }

        print!(
            "{:15}{}: {} ",
            format!("[problem {}]", problem.id),
            solver.name(),
            solution_meta.summarize()
        );

        match (&best_sol, &new_best_sol) {
            // new best
            (Some((_, best_sol)), true) => {
                let improvement = best_sol.total_score - solution_meta.total_score;
                println!(
                    "!!! WE ARE WINNING SON !!!, improvement of {}! previous best: {}",
                    improvement,
                    best_sol.summarize()
                );
            }
            // nothing special, no new best
            (Some((_, best_sol)), false) => {
                println!("lower than best: {}", best_sol.summarize());
            }
            // first solution ever
            (None, _) => {
                println!("!!! FIRST BLOOD !!!");
            }
        }
    }
    Ok(())
}

fn solve(
    input_moves: Option<Vec<Move>>,
    solvers: &[String],
    problem_paths: &[PathBuf],
) -> std::io::Result<()> {
    let base_solution_dir = PathBuf::from("./solutions/");

    let solvers: Vec<_> = solvers
        .iter()
        .map(|solver_name| create_solver(input_moves.clone(), solver_name))
        .collect();

    problem_paths
        .par_iter()
        .map(|problem_path| solve_problem(&solvers, &base_solution_dir, problem_path))
        .collect::<std::io::Result<()>>()
}

pub fn default_command(
    input_moves: Option<Vec<Move>>,
    problem_paths: &[PathBuf],
    solvers: Option<Vec<String>>,
) -> Result<(), std::io::Error> {
    match (problem_paths, solvers) {
        ([problem_path], None) => {
            gui_main(input_moves, &std::path::PathBuf::from(problem_path));
            Ok(())
        }
        (paths, Some(solvers)) => solve(input_moves, &solvers, paths),
        (_, None) => panic!("No problem paths and solvers provided"),
    }
}
