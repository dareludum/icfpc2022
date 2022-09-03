use crate::block::{Block, BlockId};
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

    if block_a.rect().width() != block_b.rect().width()
        || block_a.rect().height() != block_b.rect().height()
    {
        return Err(MoveError::InvalidInput(format!(
            "Blocks are not the same size, [{}] has size [{},{}] while [{}] has size [{},{}]",
            block_a_id,
            block_a.rect().width(),
            block_a.rect().height(),
            block_b_id,
            block_b.rect().width(),
            block_b.rect().height(),
        )));
    }

    if let (&mut Block::Simple(ref mut b_a), &mut Block::Simple(ref mut b_b)) =
        (&mut block_a, &mut block_b)
    {
        std::mem::swap(&mut b_a.c, &mut b_b.c);
    } else {
        todo!("Swap for complex blocks is not implemented")
    }

    canvas.put_block(block_a);
    canvas.put_block(block_b);
    Ok((
        cost,
        UndoMove::swap(canvas, block_a_id.clone(), block_b_id.clone()),
    ))
}
