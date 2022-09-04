use crate::{
    block::Block,
    canvas::Canvas,
    moves::{AppliedMove, Cost, Move, MoveType, Orientation, UndoMoveOp},
    painting::Painting,
};

use super::Solver;

pub struct Annealing {
    pub step: u32,
}

impl Solver for Annealing {
    fn name(&self) -> &'static str {
        if self.step == 10 {
            "annealing"
        } else if self.step == 4 {
            "annealing_s4"
        } else {
            todo!("Implement this")
        }
    }

    fn solve_core(&self, canvas: &mut Canvas, painting: &Painting) -> Vec<AppliedMove> {
        let block = canvas.get_block(&"0".into()).unwrap();
        let best_avg_color = painting.calculate_average_color(block.rect());
        let static_move = Move::Color("0".into(), best_avg_color);
        let applied_static_move = static_move.apply(canvas).unwrap();

        let mut applied_moves = vec![];
        let mut current_move_cost = Cost(0);
        let mut current_painting_score = painting.calculate_score_canvas(canvas);
        const KMAX: u32 = 5000;
        for k in 0..KMAX {
            let t = self.temperature(1.0 - (k as f32 + 1.0) / KMAX as f32);
            let budget = (current_painting_score.0 - current_move_cost.0) as i64;
            let mut iteration_canvas = canvas.clone();
            let iteration_moves = self.pick_neighbor(
                &mut iteration_canvas,
                painting,
                applied_moves.clone(),
                budget,
            );
            let new_painting_score = painting.calculate_score_canvas(&iteration_canvas);
            let new_move_cost = iteration_moves.iter().map(|am| am.cost).sum::<Cost>();
            let e_curr = (current_painting_score + current_move_cost).0 as f32;
            let e_new = (new_painting_score + new_move_cost).0 as f32;
            if self.p(e_curr, e_new, t) >= rand::random::<f32>() {
                *canvas = iteration_canvas;
                applied_moves = iteration_moves;
                current_move_cost = new_move_cost;
                current_painting_score = new_painting_score;
            }
        }
        applied_moves.insert(0, applied_static_move);
        applied_moves
    }
}

impl Annealing {
    fn temperature(&self, x: f32) -> f32 {
        x
    }

    fn pick_neighbor(
        &self,
        canvas: &mut Canvas,
        painting: &Painting,
        mut moves: Vec<AppliedMove>,
        budget: i64,
    ) -> Vec<AppliedMove> {
        let viable_moves = self.get_viable_moves(canvas, &moves, budget);
        let (undo_count, mov) = &viable_moves[rand::random::<usize>() % viable_moves.len()];

        for _ in 0..*undo_count {
            let am = moves.pop().unwrap();
            am.undo(canvas);
        }
        let applied_move = mov.clone().apply(canvas).expect("Failed to apply a move");
        let undo_op = applied_move.undo.operation.clone();
        moves.push(applied_move);

        // Now automatically color if it makes sense to do so
        match undo_op {
            UndoMoveOp::Cut {
                delete_block_ids, ..
            } => {
                for b_id in delete_block_ids {
                    let b = canvas.get_block(&b_id).unwrap();
                    let before = painting.calculate_score_canvas(canvas);
                    let color = painting.calculate_average_color(b.rect());
                    let mov = Move::Color(b_id, color);
                    let am = mov.apply(canvas).unwrap();
                    let after = painting.calculate_score_canvas(canvas);
                    if (after.0 + am.cost.0) > before.0 {
                        am.undo(canvas);
                    } else {
                        moves.push(am);
                    }
                }
            }
            _ => panic!("Missing move handler"),
        }

        moves
    }

    fn get_viable_moves(
        &self,
        canvas: &mut Canvas,
        current_moves: &Vec<AppliedMove>,
        mut budget: i64,
    ) -> Vec<(u32, Move)> {
        let mut moves = vec![];
        if budget > 0 {
            for b in canvas.blocks_iter() {
                self.get_moves_for_block(b, canvas, budget, &mut moves, 0);
            }
        }
        for i in 0..current_moves.len() {
            let am = &current_moves[current_moves.len() - i - 1];
            budget += am.cost.0 as i64;
            if budget > 0 {
                if let UndoMoveOp::Cut { restore_blocks, .. } = &am.undo.operation {
                    for b in restore_blocks {
                        self.get_moves_for_block(b, canvas, budget, &mut moves, i as u32 + 1);
                    }
                }
            }
        }
        moves
    }

    fn get_moves_for_block(
        &self,
        b: &Block,
        canvas: &Canvas,
        budget: i64,
        moves: &mut Vec<(u32, Move)>,
        undo_count: u32,
    ) {
        let step: u32 = self.step;
        let xstep = self.step * 2;

        let r = b.rect();
        let linear_cut_cost = Move::get_cost(MoveType::LineCut, r.area(), canvas.area);
        if (linear_cut_cost.0 as i64) < budget {
            for x in (step..r.width() - 1).step_by(step as usize) {
                moves.push((
                    undo_count,
                    Move::LineCut(b.get_id().clone(), Orientation::Vertical, r.x() + x),
                ));
            }
            for y in (step..r.height() - 1).step_by(step as usize) {
                moves.push((
                    undo_count,
                    Move::LineCut(b.get_id().clone(), Orientation::Horizontal, r.y() + y),
                ));
            }
        }
        let cross_cut_cost = Move::get_cost(MoveType::PointCut, r.area(), canvas.area);
        if (cross_cut_cost.0 as i64) < budget {
            for x in (xstep..r.width() - 1).step_by(xstep as usize) {
                for y in (xstep..r.height() - 1).step_by(xstep as usize) {
                    moves.push((
                        undo_count,
                        Move::PointCut(b.get_id().clone(), r.x() + x, r.y() + y),
                    ));
                }
            }
        }
    }

    fn p(&self, e_curr: f32, e_new: f32, t: f32) -> f32 {
        if e_new < e_curr {
            1.0
        } else {
            ((-(e_new - e_curr)) / t).exp()
        }
    }
}
