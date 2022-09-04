use crate::block::{BlockData, BlockId};
use crate::canvas::Canvas;
use crate::moves::{Cost, Move, MoveError, UndoMove};

pub fn swap(
    mov: &Move,
    canvas: &mut Canvas,
    block_a_id: &BlockId,
    block_b_id: &BlockId,
) -> Result<(Cost, UndoMove), MoveError> {
    let mut block_a = canvas.remove_move_block(block_a_id)?;
    let mut block_b = canvas.remove_move_block(block_b_id)?;

    let cost = Cost::compute(mov, block_a.size(), canvas.area);

    if block_a.r.width() != block_b.r.width() || block_a.r.height() != block_b.r.height() {
        return Err(MoveError::InvalidInput(format!(
            "Blocks are not the same size, [{}] has size [{},{}] while [{}] has size [{},{}]",
            block_a_id,
            block_a.r.width(),
            block_a.r.height(),
            block_b_id,
            block_b.r.width(),
            block_b.r.height(),
        )));
    }

    let x_diff = block_a
        .r
        .bottom_left
        .x
        .wrapping_sub(block_b.r.bottom_left.x);
    let y_diff = block_a
        .r
        .bottom_left
        .y
        .wrapping_sub(block_b.r.bottom_left.y);

    std::mem::swap(&mut block_a.data, &mut block_b.data);
    if let BlockData::Complex(bs) = &mut block_a.data {
        for b in bs {
            b.r.bottom_left.x = b.r.bottom_left.x.wrapping_add(x_diff);
            b.r.bottom_left.y = b.r.bottom_left.y.wrapping_add(y_diff);
            b.r.top_right.x = b.r.top_right.x.wrapping_add(x_diff);
            b.r.top_right.y = b.r.top_right.y.wrapping_add(y_diff);
        }
    }
    if let BlockData::Complex(bs) = &mut block_b.data {
        for b in bs {
            b.r.bottom_left.x = b.r.bottom_left.x.wrapping_sub(x_diff);
            b.r.bottom_left.y = b.r.bottom_left.y.wrapping_sub(y_diff);
            b.r.top_right.x = b.r.top_right.x.wrapping_sub(x_diff);
            b.r.top_right.y = b.r.top_right.y.wrapping_sub(y_diff);
        }
    }

    canvas.put_block(block_a);
    canvas.put_block(block_b);
    Ok((
        cost,
        UndoMove::swap(canvas, block_a_id.clone(), block_b_id.clone()),
    ))
}
