use crate::{
    canvas::Canvas,
    moves::{AppliedMove, Move, Orientation, UndoMoveOp},
    painting::Painting,
    solvers::Processor,
};

pub struct Shake;

impl Processor for Shake {
    fn name(&self) -> &str {
        "shake"
    }

    fn process(
        &self,
        applied_moves: &mut Vec<AppliedMove>,
        canvas: &mut Canvas,
        painting: &Painting,
    ) {
        let mut base_canvas = canvas.clone();
        self.reset_canvas(&mut base_canvas, applied_moves);

        loop {
            let mut updated = false;
            for i in 0..applied_moves.len() {
                let moves_copy = applied_moves.clone();
                let am = &mut applied_moves[i];
                let mut options = vec![];
                if let Move::LineCut(_, o, ref mut offset) = &mut am.mov {
                    let curr = *offset;
                    if let UndoMoveOp::Cut { restore_blocks, .. } = &am.undo.operation {
                        let orig_block = &restore_blocks[0];
                        if curr < orig_block.r.x() + 3
                            || curr >= (orig_block.r.x() + orig_block.r.width()) - 3
                            || curr < orig_block.r.y() + 3
                            || curr >= (orig_block.r.y() + orig_block.r.height()) - 3
                        {
                            continue; // Avoid overflows
                        }

                        match o {
                            Orientation::Horizontal => {
                                for v in curr - 2..=(curr - 1) {
                                    options.push(v);
                                }
                                for v in curr + 1..=(curr + 2) {
                                    options.push(v);
                                }
                            }
                            Orientation::Vertical => {
                                for v in curr - 2..=(curr - 1) {
                                    options.push(v);
                                }
                                for v in curr + 1..=(curr + 2) {
                                    options.push(v);
                                }
                            }
                        }
                    }
                }

                let mut best_score = painting.calculate_score_canvas(canvas);
                'options: for off in options {
                    let mut new_moves = moves_copy.clone();
                    match &mut new_moves[i].mov {
                        Move::LineCut(_, _, ref mut offset) => {
                            *offset = off;
                        }
                        _ => unreachable!(),
                    }

                    let mut attempt_canvas = base_canvas.clone();
                    let mut new_applied_moves = vec![];
                    for am in new_moves {
                        match am.mov.apply(&mut attempt_canvas) {
                            Ok(am) => new_applied_moves.push(am),
                            Err(_) => continue 'options, // Why is this happening?
                        }
                    }
                    let score = painting.calculate_score_canvas(&attempt_canvas);
                    if score.0 < best_score.0 {
                        // println!("Updating score: {} -> {}", best_score.0, score.0);
                        best_score = score;
                        *applied_moves = new_applied_moves;
                        *canvas = attempt_canvas;
                        updated = true;
                    }
                }
            }
            if !updated {
                break;
            }
        }
    }
}

impl Shake {
    fn reset_canvas(&self, canvas: &mut Canvas, applied_moves: &[AppliedMove]) {
        for am in applied_moves.iter().rev() {
            am.clone().undo(canvas);
        }
    }
}
