use crate::{
    block::{BlockId, Point},
    canvas::Canvas,
    color::Color,
    moves::{AppliedMove, Cost, Move},
    painting::Painting,
};

use super::{Solution, Solver};

pub struct DivideConquerSolver {}

struct Params {
    max_move_cost: u64,
}

impl Solver for DivideConquerSolver {
    fn name(&self) -> &'static str {
        "divide_conquer"
    }

    fn solve(&self, canvas: &mut Canvas, painting: &Painting) -> Solution {
        let mut applied_moves = vec![];
        let mut cost = Cost(u64::MAX);
        let mut result = None;
        let mut max_move_cost = 100;

        while max_move_cost <= 1000 {
            let mut canvas = canvas.clone();
            let mut iteration_moves = vec![];
            let mut iteration_cost = Cost(0);
            self.solve_block(
                &Params { max_move_cost },
                &mut canvas,
                painting,
                &"0".to_owned(),
                &mut iteration_moves,
                &mut iteration_cost,
            );
            if iteration_cost.0 < cost.0 {
                applied_moves = iteration_moves;
                cost = iteration_cost;
                result = Some(canvas.render());
            }
            max_move_cost += 100;
        }
        let mut moves = vec![];
        for am in applied_moves {
            moves.push(am.mov);
        }
        Solution {
            result: result.unwrap(),
            moves,
            cost,
        }
    }

    fn solve_core(&self, _canvas: &mut Canvas, _painting: &Painting) -> Vec<AppliedMove> {
        panic!("API users must call solve() instead")
    }
}

impl DivideConquerSolver {
    fn solve_block(
        &self,
        params: &Params,
        canvas: &mut Canvas,
        painting: &Painting,
        id: &BlockId,
        moves: &mut Vec<AppliedMove>,
        cost: &mut Cost,
    ) {
        const SMALLEST_SIZE: u32 = 4;

        let block = canvas
            .get_block(id)
            .expect(&format!("DivideConquerSolver: Can't get block {id}"));
        let r = block.rect();
        // This is recalculated twice, essentially :(
        let counts = painting.count_colors(r);

        if counts.len() == 1 || r.width() < SMALLEST_SIZE || r.height() < SMALLEST_SIZE {
            let best_color = Color::find_average(&counts);
            let mov = Move::Color(id.to_owned(), best_color);
            let applied_move = mov.apply(canvas).unwrap();
            *cost += applied_move.cost;
            moves.push(applied_move);
            return;
        }

        let Point { x, y } = r.center();

        // TODO: assess move cost before performing it
        let cut = Move::PointCut(id.to_owned(), x, y);
        let applied_move = cut.apply(canvas).unwrap();
        if applied_move.cost.0 > params.max_move_cost {
            applied_move.undo(canvas);
            let best_color = Color::find_average(&counts);
            let mov = Move::Color(id.to_owned(), best_color);
            let applied_move = mov.apply(canvas).unwrap();
            *cost += applied_move.cost;
            moves.push(applied_move);
            return;
        }
        *cost += applied_move.cost;

        moves.push(applied_move);

        let id0 = canvas.hit_test(x, y);
        let id1 = canvas.hit_test(x - 1, y);
        let id2 = canvas.hit_test(x, y - 1);
        let id3 = canvas.hit_test(x - 1, y - 1);

        self.solve_block(params, canvas, painting, &id0, moves, cost);
        self.solve_block(params, canvas, painting, &id1, moves, cost);
        self.solve_block(params, canvas, painting, &id2, moves, cost);
        self.solve_block(params, canvas, painting, &id3, moves, cost);
    }
}
