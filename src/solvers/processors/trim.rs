use crate::{solvers::Processor, moves::{AppliedMove, Move}, canvas::Canvas, painting::Painting};

#[derive(Clone)]
pub struct Trim;

impl Processor for Trim {
    fn name(&self) -> &str {
        "trim"
    }

    fn process(
        &self,
        applied_moves: &mut Vec<AppliedMove>,
        canvas: &mut Canvas,
        _painting: &Painting,
    ) {
        for i in 0..applied_moves.len() {
            let len = applied_moves.len();
            let am = &mut applied_moves[len - i - 1];
            match am.mov {
                Move::Color(_, _) | Move::Swap(_, _) => break,
                Move::LineCut(_, _, _) | Move::PointCut(_, _, _) | Move::Merge(_, _) => {
                    applied_moves.pop().unwrap().undo(canvas);
                }
            }
        }
    }
}
