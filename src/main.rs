use clap::Parser;

use gui::gui_main;
use painting::Painting;
use solvers::{create_solver, SOLVERS};

mod block;
mod canvas;
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
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    if args.batch {
        for file in std::fs::read_dir("./problems")? {
            let problem_path = file?.path();
            let mut filename = problem_path.file_stem().unwrap().to_owned();
            filename.push(".txt");

            println!("Processing {:?}", problem_path);
            let painting = Painting::load(&problem_path);

            for solver_name in SOLVERS {
                let solver = create_solver(solver_name);

                let mut solution_path = std::path::PathBuf::from("./solutions/");
                solution_path.push(solver.name());
                std::fs::create_dir_all(&solution_path)?;
                solution_path.push(&filename);

                let moves = solver.solve(&painting);
                program::write_to_file(&solution_path, &moves);
            }
        }
    } else {
        let problem = match std::env::args().nth(1) {
            Some(path) => path,
            None => "./problems/3.png".to_owned(),
        };
        gui_main(&std::path::PathBuf::from(problem));
    }
    Ok(())
}
