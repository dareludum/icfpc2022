use crate::block::{BlockData, BlockId};
use crate::canvas::Canvas;
use crate::color::Color;
use crate::moves::{Block, Cost, MoveError, MoveType, UndoMove};

pub fn color(
    canvas: &mut Canvas,
    block_id: &BlockId,
    new_color: Color,
) -> Result<(Cost, UndoMove), MoveError> {
    let block = canvas.get_move_block(block_id)?;
    let cost = canvas.compute_cost(MoveType::Color, block.area());
    let block = canvas.get_move_block_mut(block_id)?;
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
