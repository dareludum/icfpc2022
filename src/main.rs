use gui::gui_main;
use raylib::prelude::*;

mod block;
mod canvas;
mod gui;
mod moves;
mod painting;

fn main() {
    let problem = match std::env::args().nth(1) {
        Some(path) => path,
        None => "./problems/3.png".to_owned(),
    };
    gui_main(&std::path::PathBuf::from(problem));
}
