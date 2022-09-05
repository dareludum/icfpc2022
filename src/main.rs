#![feature(get_mut_unchecked)]

extern crate derive_more;
extern crate nalgebra as na;
extern crate nom;

use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::{ffi::OsString, fs::DirEntry, path::PathBuf};

use clap::Parser;
use cmd::default::*;
use cmd::stats::*;
use cmd::Args;
use cmd::Commands;
use helpers::*;
use moves::Move;
use solvers::SOLVERS;

use crate::parser::parse_move_line;

mod block;
mod canvas;
mod cmd;
mod color;
mod dto;
mod gui;
mod helpers;
mod moves;
mod painting;
mod parser;
mod program;
mod solvers;

fn get_problem_paths(args: &Args, force_batch: bool) -> Result<Vec<PathBuf>, std::io::Error> {
    if !args.problems.is_empty() {
        Ok(args
            .problems
            .iter()
            .map(|p| PathBuf::from(format!("./problems/{p}.png")))
            .collect())
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
        .filter_map(|f| {
            let x = os_str_to_str(f.path().file_name());
            if x.ends_with(".png") && !x.ends_with(".source.png") {
                Some(f.path())
            } else {
                None
            }
        })
        .collect();

    Ok(paths)
}

fn get_solvers(args: &Args) -> Option<Vec<String>> {
    if !args.solvers.is_empty() {
        Some(args.solvers.clone())
    } else if args.batch {
        Some(SOLVERS.iter().map(|s| s.to_string()).collect())
    } else {
        None
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

fn parse_input_moves(file_path: &String) -> std::io::Result<Vec<Move>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut moves = vec![];
    for line_res in reader.lines() {
        let line = line_res?;
        let res = parse_move_line(line.as_str());
        match res {
            Ok(("", mov)) => moves.push(mov),
            Ok((remainder, _)) => {
                panic!("parser finished before the end of the line: {line}, {remainder}")
            }
            Err(err) => panic!("failed to parse line {line:?}: {err}"),
        }
    }
    Ok(moves)
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
            let input_moves = match &args.input_moves {
                Some(input_moves_path) => Some(parse_input_moves(input_moves_path)?),
                None => None,
            };
            let problem_paths = get_problem_paths(&args, false)?;
            default_command(input_moves, &problem_paths, solvers)
        }
    }
}
