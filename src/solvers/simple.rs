use std::collections::HashMap;

use crate::{
    block::{Block, BlockId},
    canvas::Canvas,
    moves::{AppliedMove, Cost, Move, MoveType, Orientation, UndoMoveOp},
    painting::Painting,
};

use super::Solver;

pub struct Simple {
    pub allow_cross_cut: bool,
    pub step_1: bool,
}

impl Solver for Simple {
    fn name(&self) -> &'static str {
        if !self.step_1 {
            if self.allow_cross_cut {
                "simple"
            } else {
                "simple_no_x"
            }
        } else {
            if self.allow_cross_cut {
                "simple_s1"
            } else {
                "simple_no_x_s1"
            }
        }
    }

    fn solve_core(&self, canvas: &mut Canvas, painting: &Painting) -> Vec<AppliedMove> {
        let mut applied_moves = vec![];

        let mut total_move_cost = Cost(0);
        let mut current_painting_score = painting.calculate_score_canvas(canvas);

        let mut best_moves_cache: HashMap<BlockId, Option<(Move, i64)>> = HashMap::new();
        loop {
            let budget = (current_painting_score.0 - total_move_cost.0) as i64;

            let mut best_moves = vec![];
            for b in canvas.blocks_iter() {
                let mov = match best_moves_cache.get(b.get_id()) {
                    Some(v) => v.clone(),
                    None => {
                        let mov = self.get_best_move_for_block(b, canvas, painting, budget);
                        best_moves_cache.insert(b.get_id().to_owned(), mov.clone());
                        mov
                    }
                };
                if let Some(mov) = mov {
                    best_moves.push(mov);
                }
            }

            if best_moves.is_empty() {
                break;
            }

            let mov = best_moves.into_iter().min_by_key(|(_, r)| *r).unwrap().0;
            let am = mov.apply(canvas).unwrap();
            total_move_cost += am.cost;
            let undo_op = am.undo.operation.clone();
            applied_moves.push(am);

            if let UndoMoveOp::Cut {
                delete_block_ids, ..
            } = undo_op
            {
                for b_id in delete_block_ids {
                    let b = canvas.get_block(&b_id).unwrap();
                    let before = painting.calculate_score_canvas(canvas);
                    let color = painting.calculate_average_color(b.rect());
                    let mov = Move::Color(b_id.to_owned(), color);
                    let am = mov.apply(canvas).unwrap();
                    let after = painting.calculate_score_canvas(canvas);
                    if (after.0 + am.cost.0) > before.0 {
                        am.undo(canvas);
                    } else {
                        total_move_cost += am.cost;
                        applied_moves.push(am);
                    }
                }
            }

            current_painting_score = painting.calculate_score_canvas(canvas);
        }

        applied_moves
    }
}

impl Simple {
    fn get_best_move_for_block(
        &self,
        b: &Block,
        canvas: &Canvas,
        painting: &Painting,
        budget: i64,
    ) -> Option<(Move, i64)> {
        let (step, xstep) = if self.step_1 { (1, 1) } else { (2, 20) };

        let mut best_move = None;
        let mut best_result = i64::MAX;

        let r = b.rect();
        let linear_cut_cost = Move::get_cost(MoveType::LineCut, r.area(), canvas.area);
        if (linear_cut_cost.0 as i64) < budget {
            for x in (step..r.width()).step_by(step as usize) {
                let mov = Move::LineCut(b.get_id().clone(), Orientation::Vertical, r.x() + x);
                let result = self.assess_move(&mov, canvas, painting);
                if result < best_result {
                    best_result = result;
                    best_move = Some(mov);
                }
            }
            for y in (step..r.height()).step_by(step as usize) {
                let mov = Move::LineCut(b.get_id().clone(), Orientation::Horizontal, r.y() + y);
                let result = self.assess_move(&mov, canvas, painting);
                if result < best_result {
                    best_result = result;
                    best_move = Some(mov);
                }
            }
        }
        if self.allow_cross_cut {
            let cross_cut_cost = Move::get_cost(MoveType::PointCut, r.area(), canvas.area);
            if (cross_cut_cost.0 as i64) < budget {
                for x in (xstep..r.width() - 1).step_by(xstep as usize) {
                    for y in (xstep..r.height() - 1).step_by(xstep as usize) {
                        let mov = Move::PointCut(b.get_id().clone(), r.x() + x, r.y() + y);
                        let result = self.assess_move(&mov, canvas, painting);
                        if result < best_result {
                            best_result = result;
                            best_move = Some(mov);
                        }
                    }
                }
            }
        }

        if best_result > 0 {
            None
        } else {
            best_move.map(|m| (m, best_result))
        }
    }

    // Lower result is better
    fn assess_move(&self, mov: &Move, canvas: &Canvas, painting: &Painting) -> i64 {
        let mut canvas_temp = canvas.clone();
        let am = mov.clone().apply(&mut canvas_temp).unwrap();
        let mut total_move_cost = am.cost;
        if let UndoMoveOp::Cut {
            delete_block_ids, ..
        } = &am.undo.operation
        {
            for b_id in delete_block_ids {
                let b = canvas_temp.get_block(b_id).unwrap();
                let before = painting.calculate_score_canvas(&canvas_temp);
                let color = painting.calculate_average_color(b.rect());
                let mov = Move::Color(b_id.to_owned(), color);
                let am = mov.apply(&mut canvas_temp).unwrap();
                let after = painting.calculate_score_canvas(&canvas_temp);
                if (after.0 + am.cost.0) > before.0 {
                    am.undo(&mut canvas_temp);
                } else {
                    total_move_cost += am.cost;
                }
            }
        }
        let before = painting.calculate_score_canvas(canvas);
        let after = painting.calculate_score_canvas(&canvas_temp);
        let improvement = after.0 as i64 - before.0 as i64;
        total_move_cost.0 as i64 + improvement
    }
}
