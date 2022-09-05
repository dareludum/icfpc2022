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
        // This code doesn't seem to properly trim :/
        if !applied_moves.is_empty() {
            for i in (0..applied_moves.len() - 1).rev() {
                let am = &mut applied_moves[i];
                match am.mov {
                    Move::Color(_, _) | Move::Swap(_, _) => break,
                    Move::LineCut(_, _, _) | Move::PointCut(_, _, _) | Move::Merge(_, _) => {
                        applied_moves.pop().unwrap().undo(canvas);
                    }
                }
            }
        }
    }
}
