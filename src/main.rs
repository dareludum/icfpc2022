extern crate derive_more;

use std::{
    ffi::OsStr,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};

use clap::Parser;

use gui::gui_main;
use painting::Painting;
use rayon::prelude::*;
use solvers::{create_solver, SOLVERS};

use crate::{canvas::Canvas, dto::SolvedSolutionDto};

mod block;
mod canvas;
mod color;
mod dto;
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

fn solve(solvers: &[String], problem_paths: &[&Path]) -> std::io::Result<()> {
    let base_solution_dir = PathBuf::from("./solutions/");

    problem_paths
        .par_iter()
        .map(|problem_path| {
            let problem_num = problem_path.file_stem().unwrap().to_str().unwrap();
            let painting = Painting::load(problem_path);
            let SolvedSolutionDto {
                total_score: current_best_total,
                solver_name: current_best_solver,
                ..
            } = read_current_best(&base_solution_dir, problem_num)?
                .unwrap_or_else(SolvedSolutionDto::not_solved);

            for solver_name in solvers {
                let solver = create_solver(solver_name);

                let current_solution_dir = &base_solution_dir.join("current").join(solver.name());
                std::fs::create_dir_all(current_solution_dir)?;

                let initial_config_path = problem_path.with_extension("json");
                let mut canvas = Canvas::try_create(initial_config_path, &painting)?;
                let solution = solver.solve(&mut canvas, &painting);
                let isl_path = current_solution_dir.join(problem_num).with_extension("txt");

                program::write_to_file(&isl_path, &solution.moves)?;
                solution
                    .result
                    .write_to_file(&current_solution_dir.join(problem_num).with_extension("png"));

                let score = painting.calculate_score(&solution.result);
                let total = score + solution.cost;

                let solution_meta = SolvedSolutionDto {
                    solver_name: solver.name().to_string(),
                    score: score.0,
                    total_score: total.0,
                    solution_cost: solution.cost.0,
                };

                let solution_meta_json = serde_json::to_string_pretty(&solution_meta)?;
                std::fs::write(
                    isl_path.with_file_name(format!("{problem_num}_meta.json")),
                    solution_meta_json,
                )?;

                write_best(&base_solution_dir, problem_num, &solution_meta)?;

                println!(
                    "{:15}{}: {} ({} + {}); best: {} {} {}",
                    format!("[problem {}]", problem_num),
                    solver.name(),
                    total.0,
                    score.0,
                    solution.cost.0,
                    current_best_solver,
                    current_best_total,
                    win_indicator_str(current_best_total, solution_meta.total_score)
                );
            }

            Ok(())
        })
        .collect::<std::io::Result<()>>()
}

fn win_indicator_str(old: u64, new: u64) -> String {
    if new < old {
        "!!! WE ARE WINNING SON !!!".to_string()
    } else {
        "".to_string()
    }
}

fn get_problem_paths(args: &Args) -> Result<Vec<PathBuf>, std::io::Error> {
    if let Some(problem) = args.problem.clone() {
        Ok(vec![PathBuf::from(&problem)])
    } else if args.batch {
        let paths: Vec<PathBuf> = std::fs::read_dir("./problems")?
            .collect::<Result<Vec<DirEntry>, _>>()?
            .iter()
            .filter_map(|f| match f.path().extension() {
                Some(ext) if ext == OsStr::new("png") => Some(f.path()),
                _ => None,
            })
            .collect();

        Ok(paths)
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
        (paths, Some(solvers)) => solve(&solvers, paths),
        (_, None) => panic!("No problem paths and solvers provided"),
    }
}
