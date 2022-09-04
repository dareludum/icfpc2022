use crate::block::{Block, BlockId, Rect, SubBlock};
use crate::canvas::Canvas;
use crate::moves::{Cost, MoveError, MoveType, UndoMove};

pub fn merge(
    canvas: &mut Canvas,
    block_a_id: &BlockId,
    block_b_id: &BlockId,
) -> Result<(Cost, UndoMove), MoveError> {
    let block_a = canvas.get_move_block(block_a_id)?;
    let block_b = canvas.get_move_block(block_b_id)?;
    let cost = Cost::compute(
        MoveType::Merge,
        std::cmp::max(block_a.size(), block_b.size()),
        canvas.area,
    );
    let a_bottom_left = block_a.r.bottom_left;
    let b_bottom_left = block_b.r.bottom_left;
    let a_top_right = block_a.r.top_right;
    let b_top_right = block_b.r.top_right;
    drop(block_a);
    drop(block_b);

    // vertical merge
    if (a_bottom_left.y == b_top_right.y || a_top_right.y == b_bottom_left.y)
        && a_bottom_left.x == b_bottom_left.x
        && a_top_right.x == b_top_right.x
    {
        let block_a = canvas.remove_block(block_a_id).unwrap();
        let block_b = canvas.remove_block(block_b_id).unwrap();
        let (new_bottom_left, new_top_right) = if a_bottom_left.y < b_bottom_left.y {
            (a_bottom_left, b_top_right)
        } else {
            (b_bottom_left, a_top_right)
        };
        let new_id = canvas.next_merge_id();
        let undo = UndoMove::merge(canvas, new_id.clone(), block_a.clone(), block_b.clone());
        let mut children: Vec<SubBlock> = vec![];
        children.extend(block_a.take_children().into_iter());
        children.extend(block_b.take_children().into_iter());
        canvas.put_block(Block::new_complex(
            new_id,
            Rect::new(new_bottom_left, new_top_right),
            children,
        ));
        return Ok((cost, undo));
    }

    // horizontal merge
    if (b_top_right.x == a_bottom_left.x || a_top_right.x == b_bottom_left.x)
        && a_bottom_left.y == b_bottom_left.y
        && a_top_right.y == b_top_right.y
    {
        let block_a = canvas.remove_block(block_a_id).unwrap();
        let block_b = canvas.remove_block(block_b_id).unwrap();
        let (new_bottom_left, new_top_right) = if a_bottom_left.x < b_bottom_left.x {
            (a_bottom_left, b_top_right)
        } else {
            (b_bottom_left, a_top_right)
        };
        let new_id = canvas.next_merge_id();
        let undo = UndoMove::merge(canvas, new_id.clone(), block_a.clone(), block_b.clone());
        let mut children: Vec<SubBlock> = vec![];
        children.extend(block_a.take_children().into_iter());
        children.extend(block_b.take_children().into_iter());
        canvas.put_block(Block::new_complex(
            new_id,
            Rect::new(new_bottom_left, new_top_right),
            children,
        ));
        return Ok((cost, undo));
    }

    Err(MoveError::LogicError(format!(
        "Blocks [{}] and [{}] are not mergable",
        block_a_id, block_b_id
    )))
}
