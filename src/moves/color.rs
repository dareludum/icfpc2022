use crate::block::{BlockData, BlockId};
use crate::canvas::Canvas;
use crate::color::Color;
use crate::moves::{Block, Cost, Move, MoveError, UndoMove};

pub fn color(
    mov: &Move,
    canvas: &mut Canvas,
    block_id: &BlockId,
    new_color: Color,
) -> Result<(Cost, UndoMove), MoveError> {
    let canvas_area = canvas.area;
    let block = canvas.get_move_block_mut(block_id)?;
    let cost = Cost::compute(mov, block.size(), canvas_area);
    let (block_id, rect) = match block.data {
        // if the block is simple, change its color
        BlockData::Simple(ref mut c) => {
            let old_color = *c;
            *c = new_color;
            return Ok((
                cost,
                UndoMove::simple_color(canvas, block_id.clone(), old_color),
            ));
        }
        // if its complex, turn it into a simple block
        BlockData::Complex(_) => (block.id.clone(), block.r),
    };
    let old_block = block.clone();
    *block = Block::new_simple(block_id, rect, new_color);
    Ok((cost, UndoMove::complex_color(canvas, old_block)))
}
