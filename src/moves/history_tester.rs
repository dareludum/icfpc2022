use crate::canvas::Canvas;
use crate::moves::{Cost, Move, MoveError, UndoMove};

pub struct HistoryTester {
    canvas: Canvas,
    cost: Cost,
    history: Vec<(Move, Canvas, UndoMove)>,
}

impl HistoryTester {
    pub fn new(canvas: Canvas) -> Self {
        HistoryTester {
            canvas,
            cost: Cost(0),
            history: vec![],
        }
    }

    pub fn apply(&mut self, mov: Move) -> Result<(), MoveError> {
        let ref_canvas = self.canvas.clone();
        let (mov_cost, mov_undo) = mov.apply(&mut self.canvas)?;
        self.cost += mov_cost;
        self.history.push((mov, ref_canvas, mov_undo));
        Ok(())
    }

    pub fn validate_history(self) {
        let mut cur_canvas = self.canvas;
        for (mov, ref_canvas, mov_undo) in self.history.into_iter().rev() {
            mov_undo.clone().apply(&mut cur_canvas);
            assert_eq!(&ref_canvas, &cur_canvas, "failed to undo {:?}", mov);
        }
    }

    pub fn get_canvas(&self) -> &Canvas {
        &self.canvas
    }

    pub fn get_cost(&self) -> Cost {
        self.cost
    }
}
