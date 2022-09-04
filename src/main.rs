extern crate derive_more;

use std::{
    ffi::{OsStr, OsString},
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};

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
    problem: Option<u8>,
    #[clap(short, long)]
    solver: Option<String>,
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Stats,
}

fn os_str_to_str(str: Option<&OsStr>) -> String {
    str.expect("OsStr is None")
        .to_str()
        .expect("Can't convert OsStr to String")
        .to_string()
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

fn solve(solvers: &[String], problem_paths: &[PathBuf]) -> std::io::Result<()> {
    let base_solution_dir = PathBuf::from("./solutions/");

    problem_paths
        .par_iter()
        .map(|problem_path| {
            let problem_num = os_str_to_str(problem_path.file_stem());
            let painting = Painting::load(problem_path);
            let SolvedSolutionDto {
                total_score: current_best_total,
                solver_name: current_best_solver,
                ..
            } = read_current_best(&base_solution_dir, &problem_num)?
                .unwrap_or_else(SolvedSolutionDto::not_solved);

            for solver_name in solvers {
                let solver = create_solver(solver_name);

                let current_solution_dir = &base_solution_dir.join("current").join(solver.name());
                std::fs::create_dir_all(current_solution_dir)?;

                let initial_config_path = problem_path.with_extension("json");
                let mut canvas = Canvas::try_create(initial_config_path, &painting)?;
                let solution = solver.solve(&mut canvas, &painting);
                let isl_path = current_solution_dir
                    .join(&problem_num)
                    .with_extension("txt");

                program::write_to_file(&isl_path, &solution.moves)?;
                solution.result.write_to_file(
                    &current_solution_dir
                        .join(&problem_num)
                        .with_extension("png"),
                );

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

                write_best(&base_solution_dir, &problem_num, &solution_meta)?;

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

fn get_problem_paths(args: &Args, force_batch: bool) -> Result<Vec<PathBuf>, std::io::Error> {
    if let Some(problem) = args.problem {
        Ok(vec![PathBuf::from(format!("./problems/{problem}.png"))])
    } else if args.batch || force_batch {
        Ok(get_all_problem_paths()?)
    } else {
        Ok(vec![PathBuf::from("./problems/3.png")])
    }
}

fn get_all_problem_paths() -> Result<Vec<PathBuf>, std::io::Error> {
    let paths: Vec<PathBuf> = std::fs::read_dir("./problems")?
        .collect::<Result<Vec<DirEntry>, _>>()?
        .iter()
        .filter_map(|f| match f.path().extension() {
            Some(ext) if ext == OsStr::new("png") => Some(f.path()),
            _ => None,
        })
        .collect();

    Ok(paths)
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

fn default_command(
    problem_paths: &[PathBuf],
    solvers: Option<Vec<String>>,
) -> Result<(), std::io::Error> {
    match (problem_paths, solvers) {
        ([problem_path], None) => {
            gui_main(&std::path::PathBuf::from(problem_path));
            Ok(())
        }
        (paths, Some(solvers)) => solve(&solvers, paths),
        (_, None) => panic!("No problem paths and solvers provided"),
    }
}

fn stats(problems_n: &[String], solvers: &[String]) -> Result<(), std::io::Error> {
    let mut sum_best: u64 = 0;

    for n in problems_n {
        let best_fname = format!("./solutions/best/{n}_meta.json");
        let best_path = Path::new(&best_fname);

        let best: SolvedSolutionDto = serde_json::from_str(&fs::read_to_string(best_path)?)?;
        sum_best += best.total_score;
        let mut current_solved = Vec::with_capacity(problems_n.len());

        for solver in solvers {
            let path_s = format!("./solutions/current/{solver}/{n}_meta.json");
            let path = Path::new(&path_s);
            let current = if let Ok(true) = path.try_exists() {
                serde_json::from_str(&fs::read_to_string(path)?)?
            } else {
                SolvedSolutionDto::not_solved()
            };

            current_solved.push(current);
        }

        current_solved.sort_by_key(|x| x.total_score);

        println!("Problem {n}");
        println!("------------------------------------");
        println!("best: {} score={}", best.solver_name, best.total_score);
        current_solved
            .iter()
            .for_each(|x| println!("{} score={}", x.solver_name, x.total_score));
        println!("------------------------------------");
    }
    println!("------------------------------------");
    println!("Sum of all best: {sum_best}");

    Ok(())
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let solvers = get_solvers(&args);

    match &args.command {
        Some(Commands::Stats) => {
            let problem_paths = get_problem_paths(&args, true)?;

            let mut problems: Vec<String> = problem_paths
                .iter()
                .map(|p| os_str_to_str(p.file_stem()))
                .collect();

            problems.sort_by_key(|x| x.parse::<u8>().unwrap());
            stats(&problems, &solvers.unwrap_or_else(list_current_solvers))
        }
        _ => {
            let problem_paths = get_problem_paths(&args, false)?;
            default_command(&problem_paths, solvers)
        }
    }
}

fn list_current_solvers() -> Vec<String> {
    let mut current_solvers = vec![];
    let solvers_dir = std::fs::read_dir(PathBuf::from("./solutions/current"))
        .expect("Can't list solutions current dir");

    for solver in solvers_dir {
        let (id_dir, file_name) = solver
            .and_then(|x| {
                let ftype = x.file_type()?;
                Ok((ftype, x.file_name()))
            })
            .map(|(typ, fname)| (typ.is_dir(), fname))
            .unwrap_or((false, OsString::new()));

        if id_dir {
            current_solvers.push(os_str_to_str(Some(&file_name)));
        }
    }

    current_solvers
}
