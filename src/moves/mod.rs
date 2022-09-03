use derive_more::{Add, AddAssign};

use std::fmt::Display;

use crate::{
    block::{Block, BlockId, ComplexBlock, SimpleBlock},
    canvas::Canvas,
    color::Color,
};

mod color;
mod cut;
mod merge;
mod swap;

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
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
pub enum UndoMove {
    Cut {
        delete_block_ids: Vec<BlockId>,
        restore_blocks: Vec<Block>,
    },
    SimpleColor {
        block_id: BlockId,
        prev_color: Color,
    },
    ComplexColor {
        old_block: Block,
    },
    Swap {
        a_id: BlockId,
        b_id: BlockId,
    },
    Merge {
        merged_block_id: BlockId,
        initial_a: Block,
        initial_b: Block,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Add, AddAssign)]
pub struct Cost(pub u64);

impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Orientation::Horizontal => write!(f, "y"),
            Orientation::Vertical => write!(f, "x"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum MoveError {
    LogicError(String),
    InvalidInput(String),
}

impl Move {
    pub fn apply(&self, canvas: &mut Canvas) -> Option<(Cost, UndoMove)> {
        let res: Result<(Cost, UndoMove), MoveError> = match *self {
            Move::LineCut(ref block, orientation, offset) => {
                self.line_cut(canvas, block, orientation, offset)
            }
            Move::PointCut(ref block, x, y) => self.point_cut(canvas, block, x, y),
            Move::Color(ref block, c) => self.color(canvas, block, c),
            Move::Swap(ref block_a, ref block_b) => self.swap(canvas, block_a, block_b),
            Move::Merge(ref block_a, ref block_b) => self.merge(canvas, block_a, block_b),
        };
        res.ok()
    }

    fn base_cost(&self) -> u32 {
        match self {
            Move::LineCut(..) => 7,
            Move::PointCut(..) => 10,
            Move::Color(..) => 5,
            Move::Swap(..) => 3,
            Move::Merge(..) => 1,
        }
    }

    fn compute_cost(&self, block_area: u32, canvas_area: u32) -> Cost {
        Cost((self.base_cost() as f64 * (canvas_area as f64 / block_area as f64)).round() as u64)
    }
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

impl UndoMove {
    pub fn apply(self, canvas: &mut Canvas) {
        match self {
            UndoMove::Cut {
                delete_block_ids: delete_blocks,
                restore_blocks,
            } => {
                for b in delete_blocks {
                    canvas.remove_block(&b);
                }
                for b in restore_blocks {
                    canvas.put_block(b);
                }
            }
            UndoMove::SimpleColor {
                block_id: block,
                prev_color,
            } => {
                let block = canvas.get_block_mut(&block).unwrap();
                if let Block::Simple(b) = block {
                    b.c = prev_color;
                } else {
                    panic!("Invalid block")
                }
            }
            UndoMove::ComplexColor { .. } => todo!(),
            UndoMove::Swap { a_id, b_id } => {
                Move::Swap(a_id, b_id).apply(canvas);
            }
            UndoMove::Merge { .. } => todo!(),
        }
    }
}
