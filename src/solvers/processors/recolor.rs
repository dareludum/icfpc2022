use crate::{
    block::BlockData,
    canvas::Canvas,
    color::Color,
    moves::{AppliedMove, Move},
    painting::Painting,
    solvers::Processor,
};

#[derive(Clone)]
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
                        let mut colors = vec![];
                        for (c, cnt) in &counts {
                            for _ in 0..*cnt {
                                colors.push(*c);
                            }
                        }
                        const EPS: f32 = 0.2;
                        const MAX_ITERATIONS: u32 = 1000;
                        let color_options = &[
                            Color::find_average(&counts),
                            Color::find_most_common(&counts),
                            Color::gmedian(&colors, EPS, MAX_ITERATIONS),
                            Color::pmedian(&colors, EPS, MAX_ITERATIONS),
                        ];
                        // dbg!(color_options);
                        *c = *color_options
                            .iter()
                            .min_by_key(|c| painting.calculate_score_rect(&b.r, **c) as i64)
                            .unwrap();
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
