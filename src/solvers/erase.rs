use crate::{
    canvas::Canvas,
    moves::{AppliedMove, Move},
    painting::Painting,
};

use super::Solver;

pub struct Erase;

impl Solver for Erase {
    fn name(&self) -> &'static str {
        "erase"
    }

    fn solve_core(&self, canvas: &mut Canvas, painting: &Painting) -> Vec<AppliedMove> {
        let mut applied_moves = vec![];

        // Detect block size
        let size = canvas.blocks_iter().next().unwrap().r.width();
        let count = canvas.width / size;

        // Hit-test and merge all blocks into one
        for x_i in 0..count {
            for y_i in 1..count {
                let b0_id = canvas.hit_test(x_i * size, y_i * size - 1);
                let b1_id = canvas.hit_test(x_i * size, y_i * size);
                let mov = Move::Merge(b0_id, b1_id);
                let am = mov.apply(canvas).unwrap();
                applied_moves.push(am);
            }
        }

        for x_i in 1..count {
            let b0_id = canvas.hit_test(x_i * size - 1, 0);
            let b1_id = canvas.hit_test(x_i * size, 0);
            let mov = Move::Merge(b0_id, b1_id);
            let am = mov.apply(canvas).unwrap();
            applied_moves.push(am);
        }

        assert!(canvas.blocks_count() == 1);

        let b = canvas.blocks_iter().next().unwrap();
        let mov = Move::Color(b.id.clone(), painting.calculate_average_color(&b.r));
        let am = mov.apply(canvas).unwrap();
        applied_moves.push(am);

        applied_moves
    }
}
