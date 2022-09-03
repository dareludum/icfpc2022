use crate::block::BlockId;
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

    std::mem::swap(block_a.get_id_mut(), block_b.get_id_mut());
    canvas.put_block(block_a);
    canvas.put_block(block_b);
    Ok((
        cost,
        UndoMove::swap(canvas, block_a_id.clone(), block_b_id.clone()),
    ))
}
