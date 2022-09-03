use crate::block::{Block, BlockId};
use crate::canvas::Canvas;
use crate::color::Color;
use crate::moves::Move;

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
            UndoMove::ComplexColor { old_block } => {
                canvas.remove_block(old_block.get_id());
                canvas.put_block(old_block);
            }
            UndoMove::Swap { a_id, b_id } => {
                Move::Swap(a_id, b_id).apply(canvas);
            }
            UndoMove::Merge {
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
    }
}
