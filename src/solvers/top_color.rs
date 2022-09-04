use crate::{
    canvas::Canvas,
    color::Color,
    moves::{AppliedMove, Move},
    painting::Painting,
};

use super::Solver;

pub struct TopColor {}

impl Solver for TopColor {
    fn name(&self) -> &'static str {
        "top_color"
    }

    fn solve_core(&self, canvas: &mut Canvas, painting: &Painting) -> Vec<AppliedMove> {
        let block = canvas.get_block(&"0".into()).unwrap();
        let colors = painting.count_colors(&block.r);
        let top_color = Color::find_most_common(&colors);

        let mov_id = "0".into();
        let mov = Move::Color(mov_id, top_color);
        let applied_move = mov
            .apply(canvas)
            .expect("TopColor solver: couldn't perform color move"); // TODO refactor to return error, it's rust or javascript after all

        vec![applied_move]
    }
}
