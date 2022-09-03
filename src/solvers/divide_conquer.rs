use crate::{
    block::{BlockId, Point},
    canvas::Canvas,
    moves::{Cost, Move},
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

    fn solve(&self, painting: &Painting) -> Solution {
        let mut moves = vec![];
        let mut cost = Cost(u64::MAX);
        let mut result = None;
        let mut max_move_cost = 100;
        while max_move_cost <= 1000 {
            let mut canvas = Canvas::new(painting.width(), painting.height());
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
                moves = iteration_moves;
                cost = iteration_cost;
                result = Some(canvas.render());
            }
            max_move_cost += 100;
        }
        Solution {
            result: result.unwrap(),
            moves,
            cost,
        }
    }

    fn solve_core(&self, _canvas: &mut Canvas, _painting: &Painting) -> (Vec<Move>, Cost) {
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
        moves: &mut Vec<Move>,
        cost: &mut Cost,
    ) {
        const SMALLEST_SIZE: u32 = 4;

        let block = canvas.get_block(id).unwrap();
        let r = block.rect();
        // This is recalculated twice, essentially :(
        let counts = painting.count_colors(r);

        if counts.len() == 1 || r.width() < SMALLEST_SIZE || r.height() < SMALLEST_SIZE {
            let best_color = counts.into_iter().max_by_key(|(_, v)| *v).unwrap().0;
            let mov = Move::Color(id.to_owned(), best_color);
            *cost += mov.apply(canvas).unwrap().0;
            moves.push(mov);
            return;
        }

        let Point { x, y } = r.center();

        // TODO: assess move cost before performing it
        let cut = Move::PointCut(id.to_owned(), x, y);
        let (move_cost, undo) = cut.apply(canvas).unwrap();
        if move_cost.0 > params.max_move_cost {
            undo.apply(canvas);
            let best_color = counts.into_iter().max_by_key(|(_, v)| *v).unwrap().0;
            let mov = Move::Color(id.to_owned(), best_color);
            *cost += mov.apply(canvas).unwrap().0;
            moves.push(mov);
            return;
        }
        *cost += move_cost;

        moves.push(cut);

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
