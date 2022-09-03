use crate::block::BlockId;
use crate::canvas::Canvas;
use crate::color::Color;
use crate::moves::{Block, Cost, Move, MoveError, SimpleBlock, UndoMove};

impl Move {
    pub fn color(
        &self,
        canvas: &mut Canvas,
        block_id: &BlockId,
        new_color: Color,
    ) -> Result<(Cost, UndoMove), MoveError> {
        let canvas_area = canvas.area;
        let block = canvas.get_move_block_mut(block_id)?;
        let cost = self.compute_cost(block.size(), canvas_area);
        let (block_id, rect) = match block {
            // if the block is simple, change its color
            Block::Simple(ref mut simple) => {
                let old_color = simple.c;
                simple.c = new_color;
                return Ok((
                    cost,
                    UndoMove::SimpleColor {
                        block_id: block_id.clone(),
                        prev_color: old_color,
                    },
                ));
            }
            // if its complex, turn it into a simple block
            Block::Complex(ref mut complex) => (complex.id.clone(), complex.r.clone()),
        };
        let old_block = block.clone();
        *block = Block::Simple(SimpleBlock::new(block_id, rect, new_color));
        Ok((cost, UndoMove::ComplexColor { old_block }))
    }
}
