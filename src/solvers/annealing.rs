use crate::{
    canvas::Canvas,
    color::Color,
    moves::{Cost, Move, MoveType, Orientation, UndoMove, AppliedMove, UndoMoveOp},
    painting::Painting,
};

use super::Solver;

pub struct Annealing;

impl Solver for Annealing {
    fn name(&self) -> &'static str {
        "annealing"
    }

    fn solve_core(&self, canvas: &mut Canvas, painting: &Painting) -> Vec<AppliedMove> {
        let mut applied_moves = vec![];
        let mut current_move_cost = Cost(0);
        let mut current_painting_score = painting.calculate_score(&canvas.render());
        const KMAX: u32 = 1000;
        for k in 0..KMAX {
            let t = self.temperature(1.0 - (k as f32 + 1.0) / KMAX as f32);
            let budget = (current_painting_score.0 - current_move_cost.0) as i64;
            let mut iteration_canvas = canvas.clone();
            let iteration_moves = self.pick_neighbor(&mut iteration_canvas, painting, applied_moves.clone(), budget);
            let new_painting_score = painting.calculate_score(&iteration_canvas.render());
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
        let viable_moves = self.get_viable_moves(canvas, painting, &moves, budget);
        let (undo_count, mov, _) = &viable_moves[rand::random::<usize>() % viable_moves.len()];

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
                    let before = painting.calculate_score(&canvas.render());
                    let counts = painting.count_colors(b.rect());
                    let color = Color::find_average(&counts);
                    let mov = Move::Color(b_id, color);
                    let am = mov.apply(canvas).unwrap();
                    let after = painting.calculate_score(&canvas.render());
                    if after.0 >= before.0 {
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
        painting: &Painting,
        current_moves: &Vec<AppliedMove>,
        mut budget: i64,
    ) -> Vec<(u32, Move, Cost)> {
        let mut moves = self.get_moves_for_budget(canvas, painting, budget, 0);
        let mut redo_stack = vec![];
        for i in 0..current_moves.len() {
            let am = &current_moves[current_moves.len() - i - 1];
            budget += am.cost.0 as i64;
            let mov = am.clone().undo(canvas);
            match mov {
                Move::LineCut(_, _, _) | Move::PointCut(_, _, _) => {
                    moves.extend(self.get_moves_for_budget(canvas, painting, budget, i as u32 + 1));
                }
                _ => {}
            }
            redo_stack.push(mov);
        }
        for mov in redo_stack.into_iter().rev() {
            mov.apply(canvas).unwrap();
        }
        moves
    }

    fn get_moves_for_budget(
        &self,
        canvas: &Canvas,
        painting: &Painting,
        budget: i64,
        undo_count: u32,
    ) -> Vec<(u32, Move, Cost)> {
        const STEP: usize = 16;

        if budget < 0 {
            return vec![];
        }
        let mut moves = vec![];
        for b in canvas.blocks_iter() {
            let r = b.rect();
            let linear_cut_cost = Move::get_cost(MoveType::LineCut, r.area(), canvas.area);
            if (linear_cut_cost.0 as i64) < budget {
                for x in (1..r.width() - 1).step_by(STEP) {
                    moves.push((
                        undo_count,
                        Move::LineCut(b.get_id().clone(), Orientation::Vertical, r.x() + x),
                        linear_cut_cost,
                    ));
                }
                for y in (1..r.height() - 1).step_by(STEP) {
                    moves.push((
                        undo_count,
                        Move::LineCut(b.get_id().clone(), Orientation::Horizontal, r.y() + y),
                        linear_cut_cost,
                    ));
                }
            }
            let cross_cut_cost = Move::get_cost(MoveType::PointCut, r.area(), canvas.area);
            if (cross_cut_cost.0 as i64) < budget {
                for x in (1..r.width() - 1).step_by(STEP) {
                    for y in (1..r.height() - 1).step_by(STEP) {
                        moves.push((
                            undo_count,
                            Move::PointCut(b.get_id().clone(), r.x() + x, r.y() + y),
                            cross_cut_cost,
                        ));
                    }
                }
            }
            // let color_cost = Move::get_cost(MoveType::Color, r.area(), canvas.area);
            // if (color_cost.0 as i64) < budget {
            //     let counts = painting.count_colors(r);
            //     moves.push((
            //         undo_count,
            //         Move::Color(b.get_id().clone(), Color::find_average(&counts)),
            //         color_cost,
            //     ));
            // }
        }
        moves
    }

    fn p(&self, e_curr: f32, e_new: f32, t: f32) -> f32 {
        if e_new < e_curr {
            1.0
        } else {
            ((-(e_new - e_curr)) / t).exp()
        }
    }
}
