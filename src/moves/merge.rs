use crate::block::BlockId;
use crate::block::Rect;
use crate::canvas::Canvas;
use crate::moves::{ComplexBlock, Cost, Move, MoveError, SimpleBlock, UndoMove};

impl Move {
    pub fn merge(
        &self,
        canvas: &mut Canvas,
        block_a_id: &BlockId,
        block_b_id: &BlockId,
    ) -> Result<(Cost, UndoMove), MoveError> {
        let block_a = canvas.remove_move_block(block_a_id)?;
        let block_b = canvas.remove_move_block(block_b_id)?;
        let cost = Cost::compute(
            self,
            std::cmp::max(block_a.size(), block_b.size()),
            canvas.area,
        );
        let a_bottom_left = block_a.rect().bottom_left;
        let b_bottom_left = block_b.rect().bottom_left;
        let a_top_right = block_a.rect().top_right;
        let b_top_right = block_b.rect().top_right;

        // vertical merge
        if (a_bottom_left.y == b_top_right.y || a_top_right.y == b_bottom_left.y)
            && a_bottom_left.x == b_bottom_left.x
            && a_top_right.x == b_top_right.x
        {
            let (new_bottom_left, new_top_right) = if a_bottom_left.y < b_bottom_left.y {
                (a_bottom_left, b_top_right)
            } else {
                (b_bottom_left, a_top_right)
            };
            let new_id = canvas.next_merge_id();
            let undo = UndoMove::Merge {
                merged_block_id: new_id.clone(),
                initial_a: block_a.clone(),
                initial_b: block_b.clone(),
            };
            let mut children: Vec<SimpleBlock> = vec![];
            children.extend(block_a.take_children().into_iter());
            children.extend(block_b.take_children().into_iter());
            canvas.put_block(
                ComplexBlock::new(new_id, Rect::new(new_bottom_left, new_top_right), children)
                    .into(),
            );
            return Ok((cost, undo));
        }

        // horizontal merge
        if (b_top_right.x == a_bottom_left.x || a_top_right.x == b_bottom_left.x)
            && a_bottom_left.y == b_bottom_left.y
            && a_top_right.y == b_top_right.y
        {
            let (new_bottom_left, new_top_right) = if a_bottom_left.x < b_bottom_left.x {
                (a_bottom_left, b_top_right)
            } else {
                (b_bottom_left, a_top_right)
            };
            let new_id = canvas.next_merge_id();
            let undo = UndoMove::Merge {
                merged_block_id: new_id,
                initial_a: block_a.clone(),
                initial_b: block_b.clone(),
            };
            let mut children: Vec<SimpleBlock> = vec![];
            children.extend(block_a.take_children().into_iter());
            children.extend(block_b.take_children().into_iter());
            let new_id = canvas.next_merge_id();
            canvas.put_block(
                ComplexBlock::new(new_id, Rect::new(new_bottom_left, new_top_right), children)
                    .into(),
            );
            return Ok((cost, undo));
        }

        Err(MoveError::LogicError(format!(
            "Blocks [{}] and [{}] are not mergable",
            block_a_id, block_b_id
        )))
    }
}
