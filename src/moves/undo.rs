use crate::block::{Block, BlockId};
use crate::canvas::Canvas;
use crate::color::Color;
use crate::moves::Move;

#[derive(Debug, Clone)]
pub struct UndoMove {
    expected_gen: u32,
    operation: UndoMoveOp,
}

#[derive(Debug, Clone)]
pub enum UndoMoveOp {
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

impl UndoMove {
    pub fn cut(
        canvas: &mut Canvas,
        delete_block_ids: Vec<BlockId>,
        restore_blocks: Vec<Block>,
    ) -> UndoMove {
        UndoMove {
            expected_gen: canvas.next_generation(),
            operation: UndoMoveOp::Cut {
                delete_block_ids,
                restore_blocks,
            },
        }
    }

    pub fn simple_color(canvas: &mut Canvas, block_id: BlockId, prev_color: Color) -> UndoMove {
        UndoMove {
            expected_gen: canvas.next_generation(),
            operation: UndoMoveOp::SimpleColor {
                block_id,
                prev_color,
            },
        }
    }

    pub fn complex_color(canvas: &mut Canvas, old_block: Block) -> UndoMove {
        UndoMove {
            expected_gen: canvas.next_generation(),
            operation: UndoMoveOp::ComplexColor { old_block },
        }
    }

    pub fn swap(canvas: &mut Canvas, a_id: BlockId, b_id: BlockId) -> UndoMove {
        UndoMove {
            expected_gen: canvas.next_generation(),
            operation: UndoMoveOp::Swap { a_id, b_id },
        }
    }

    pub fn merge(
        canvas: &mut Canvas,
        merged_block_id: BlockId,
        initial_a: Block,
        initial_b: Block,
    ) -> UndoMove {
        UndoMove {
            expected_gen: canvas.next_generation(),
            operation: UndoMoveOp::Merge {
                merged_block_id,
                initial_a,
                initial_b,
            },
        }
    }

    pub fn apply(self, canvas: &mut Canvas) {
        if self.expected_gen != canvas.generation {
            panic!("applying undo on a mismatched canvas");
        }
        match self.operation {
            UndoMoveOp::Cut {
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
            UndoMoveOp::SimpleColor {
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
            UndoMoveOp::ComplexColor { old_block } => {
                canvas.remove_block(old_block.get_id());
                canvas.put_block(old_block);
            }
            UndoMoveOp::Swap { a_id, b_id } => {
                Move::Swap(a_id, b_id).apply(canvas).ok();
            }
            UndoMoveOp::Merge {
                merged_block_id,
                initial_a,
                initial_b,
            } => {
                canvas.remove_block(&merged_block_id);
                canvas.prev_merge_id();
                canvas.put_block(initial_a);
                canvas.put_block(initial_b);
            }
        }
        canvas.prev_generation()
    }
}
