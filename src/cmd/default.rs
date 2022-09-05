use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    dto::SolvedSolutionDto,
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

fn win_indicator_str(old: u64, new: u64) -> String {
    if new < old {
        "!!! WE ARE WINNING SON !!!".to_string()
    } else {
        "".to_string()
    }
}

fn write_best(
    base_dir: &Path,
    problem_num: &str,
    solution: &SolvedSolutionDto,
) -> std::io::Result<()> {
    let target_dir = base_dir.join("best");
    let current_solution_dir = base_dir.join("current").join(solution.solver_name.clone());

    std::fs::create_dir_all(&target_dir)?;
    let best = get_best_solution(solution, base_dir, problem_num)?;
    match best {
        (_, false) => Ok(()),
        _ => copy_output(current_solution_dir, target_dir, problem_num),
    }
}

fn fcopy_to(current_dir: &Path, target_dir: &Path, filename: &str) -> std::io::Result<()> {
    let fname_from = current_dir.join(filename);
    let fname_to = target_dir.join(filename);

    std::fs::copy(fname_from, fname_to)?;

    Ok(())
}

fn copy_output(
    current_solution_dir: PathBuf,
    target_dir: PathBuf,
    problem_num: &str,
) -> Result<(), std::io::Error> {
    fcopy_to(
        &current_solution_dir,
        &target_dir,
        &format!("{problem_num}_meta.json"),
    )?;

    fcopy_to(
        &current_solution_dir,
        &target_dir,
        &format!("{problem_num}.png"),
    )?;

    fcopy_to(
        &current_solution_dir,
        &target_dir,
        &format!("{problem_num}.txt"),
    )?;

    Ok(())
}

fn get_best_solution(
    current_solution: &SolvedSolutionDto,
    base_dir: &Path,
    problem_num: &str,
) -> std::io::Result<(SolvedSolutionDto, bool)> {
    read_current_best(base_dir, problem_num).map(|best| match best {
        Some(current_best) => {
            if current_solution.total_score < current_best.total_score {
                (current_solution.clone(), true)
            } else {
                (current_best, false)
            }
        }
        None => (current_solution.clone(), true),
    })
}

fn read_current_best(
    base_dir: &Path,
    problem_num: &str,
) -> std::io::Result<Option<SolvedSolutionDto>> {
    let meta_fname = &format!("{problem_num}_meta.json");
    let best_solution_filename = base_dir.join("best").join(meta_fname);

    match best_solution_filename.try_exists() {
        Ok(false) => Ok(None),
        Ok(true) => {
            let current_best_json = fs::read_to_string(best_solution_filename)?;
            let current_best: SolvedSolutionDto =
                serde_json::from_str(&current_best_json).expect("Deserialization error");
            Ok(Some(current_best))
        }
        Err(e) => Err(e),
    }
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
