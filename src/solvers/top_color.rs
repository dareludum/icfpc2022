use crate::{
    canvas::Canvas,
    color::Color,
    moves::{AppliedMove, Move, MoveError},
    painting::Painting,
};

use super::Solver;

#[derive(Clone)]
pub struct TopColor {
    pub use_avg: bool,
}

fn apply_batch(moves: Vec<Move>, canvas: &mut Canvas) -> Result<Vec<AppliedMove>, MoveError> {
    moves.into_iter().map(|mov| mov.apply(canvas)).collect()
}

impl Solver for TopColor {
    fn name(&self) -> &'static str {
        "top_color"
    }

    fn solve_core(&self, canvas: &mut Canvas, painting: &Painting) -> Vec<AppliedMove> {
        let mut moves = Vec::with_capacity(canvas.blocks_count());

        for block in canvas.blocks_iter() {
            let colors = painting.count_colors(&block.r);
            let top_color = if self.use_avg {
                Color::find_average(&colors)
            } else {
                Color::find_most_common(&colors)
            };

            let mov = Move::Color(block.id.clone(), top_color);

            moves.push(mov);
        }

        apply_batch(moves, canvas).expect("TopColor solver: couldn't perform color move")
    }
}
