use crate::{
    block::BlockData,
    canvas::Canvas,
    color::Color,
    moves::{AppliedMove, Move},
    painting::Painting,
    solvers::Processor,
};

pub struct Recolor;

impl Processor for Recolor {
    fn name(&self) -> &str {
        "recolor"
    }

    fn process(
        &self,
        applied_moves: &mut Vec<AppliedMove>,
        canvas: &mut Canvas,
        painting: &Painting,
    ) {
        let mut new_moves = applied_moves.clone();
        for am in new_moves.iter_mut() {
            if let Move::Color(b_id, ref mut c) = &mut am.mov {
                if let Some(b) = canvas.get_block_mut(b_id) {
                    if let BlockData::Simple(_) = b.data {
                        // Assign a new color based on the current, and not initial, block size
                        let counts = painting.count_colors(&b.r);
                        let avg_color = Color::find_average(&counts);
                        let top_color = Color::find_most_common(&counts);
                        *c = if painting.calculate_score_rect(&b.r, avg_color)
                            < painting.calculate_score_rect(&b.r, top_color)
                        {
                            avg_color
                        } else {
                            top_color
                        };
                    }
                }
            }
        }
        for am in applied_moves.iter().rev() {
            am.clone().undo(canvas);
        }
        *applied_moves = new_moves
            .into_iter()
            .map(|am| am.mov.apply(canvas).unwrap())
            .collect();
    }
}
