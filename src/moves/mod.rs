use std::fmt::Display;

use crate::{
    block::{Block, BlockId, SubBlock},
    canvas::Canvas,
    color::Color,
};

#[cfg(test)]
mod tests;

mod color;
mod cost;
mod cut;
mod merge;
mod swap;
mod undo;

pub use cost::*;
pub use undo::*;

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Orientation::Horizontal => write!(f, "y"),
            Orientation::Vertical => write!(f, "x"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Move {
    LineCut(BlockId, Orientation, u32),
    PointCut(BlockId, u32, u32),
    Color(BlockId, Color),
    Swap(BlockId, BlockId),
    Merge(BlockId, BlockId),
}

#[derive(Debug, Clone)]
pub struct AppliedMove {
    pub mov: Move,
    pub cost: Cost,
    pub undo: UndoMove,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum MoveType {
    LineCut,
    PointCut,
    Color,
    Swap,
    Merge,
}

impl Canvas {
    fn get_move_block_mut(&mut self, block_id: &BlockId) -> Result<&mut Block, MoveError> {
        match self.get_block_mut(block_id) {
            Some(block) => Ok(block),
            None => Err(MoveError::LogicError(format!(
                "missing block: {}",
                block_id
            ))),
        }
    }

    fn remove_move_block(&mut self, block_id: &BlockId) -> Result<Block, MoveError> {
        match self.remove_block(block_id) {
            Some(block) => Ok(block),
            None => Err(MoveError::LogicError(format!(
                "missing block: {}",
                block_id
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MoveError {
    LogicError(String),
    InvalidInput(String),
}

impl Move {
    pub fn apply(self, canvas: &mut Canvas) -> Result<AppliedMove, MoveError> {
        use color::*;
        use cut::*;
        use merge::*;
        use swap::*;

        let (cost, undo) = match self {
            Move::LineCut(ref block, orientation, offset) => {
                line_cut(&self, canvas, block, orientation, offset)
            }
            Move::PointCut(ref block, x, y) => point_cut(&self, canvas, block, x, y),
            Move::Color(ref block, c) => color(&self, canvas, block, c),
            Move::Swap(ref block_a, ref block_b) => swap(&self, canvas, block_a, block_b),
            Move::Merge(ref block_a, ref block_b) => merge(&self, canvas, block_a, block_b),
        }?;

        Ok(AppliedMove {
            mov: self,
            cost,
            undo,
        })
    }

    #[allow(dead_code)]
    pub fn checked_apply(self, canvas: &mut Canvas) -> Result<AppliedMove, MoveError> {
        // make a copy of the canvas before the move
        let ref_canvas = canvas.clone();
        let applied_move = self.apply(canvas)?;

        // check that applying undo to the current state reverts to the previous state
        let mut cur_canvas = canvas.clone();
        applied_move.undo.clone().apply(&mut cur_canvas);
        assert_eq!(&ref_canvas, &cur_canvas, "failed to undo");
        Ok(applied_move)
    }

    // TODO: refactor this duplication :/
    pub fn get_cost(typ: MoveType, block_area: u32, canvas_area: u32) -> Cost {
        let base_cost = match typ {
            MoveType::LineCut => 7.0,
            MoveType::PointCut => 10.0,
            MoveType::Color => 5.0,
            MoveType::Swap => 3.0,
            MoveType::Merge => 1.0,
        };
        Cost((base_cost * (canvas_area as f64 / block_area as f64)).round() as u64)
    }
}

impl AppliedMove {
    pub fn undo(self, canvas: &mut Canvas) -> Move {
        self.undo.apply(canvas);
        self.mov
    }
}
