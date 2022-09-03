use std::collections::HashMap;

use crate::{
    block::{BlockId, Point},
    canvas::Canvas,
    moves::{Cost, Move},
    painting::Painting,
};

use super::Solver;

pub struct DivideConquerSolver {}

impl Solver for DivideConquerSolver {
    fn name(&self) -> &'static str {
        "divide_conquer"
    }

    fn solve_core(&self, canvas: &mut Canvas, painting: &Painting) -> (Vec<Move>, Cost) {
        let mut moves = vec![];
        let cost = self.solve_block(canvas, painting, &"0".to_owned(), &mut moves);
        (moves, cost)
    }
}

impl DivideConquerSolver {
    fn solve_block(
        &self,
        canvas: &mut Canvas,
        painting: &Painting,
        id: &BlockId,
        moves: &mut Vec<Move>,
    ) -> Cost {
        const SMALLEST_SIZE: u32 = 4;
        let block = canvas.get_block(id).unwrap();
        let r = block.rect();
        if r.width() < SMALLEST_SIZE || r.height() < SMALLEST_SIZE {
            let mut counts = HashMap::new();
            for x in r.x()..r.top_right.x {
                for y in r.y()..r.top_right.y {
                    let color = painting.get_color(x, y);
                    if let Some(v) = counts.get_mut(&color) {
                        *v += 1;
                    } else {
                        counts.insert(color, 0);
                    }
                }
            }
            let best_color = counts.into_iter().max_by_key(|(_, v)| *v).unwrap().0;

            let mov = Move::Color(id.to_owned(), best_color);
            let cost = mov.apply(canvas).unwrap().0;
            moves.push(mov);
            return cost;
        }

        let Point { x, y } = r.center();

        let cut = Move::PointCut(id.to_owned(), x, y);
        let mut cost = cut.apply(canvas).unwrap().0;
        moves.push(cut);

        let id0 = canvas.hit_test(x, y);
        let id1 = canvas.hit_test(x - 1, y);
        let id2 = canvas.hit_test(x, y - 1);
        let id3 = canvas.hit_test(x - 1, y - 1);

        cost += self.solve_block(canvas, painting, &id0, moves);
        cost += self.solve_block(canvas, painting, &id1, moves);
        cost += self.solve_block(canvas, painting, &id2, moves);
        cost += self.solve_block(canvas, painting, &id3, moves);

        cost
    }
}
