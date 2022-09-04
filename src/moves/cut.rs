use crate::block::{BlockData, BlockId, SubBlock};
use crate::block::{Point, Rect};
use crate::canvas::Canvas;
use crate::moves::{Block, Cost, MoveError, MoveType, Orientation, UndoMove};

struct UndoCutBuilder {
    delete_blocks: Vec<BlockId>,
    restore_blocks: Vec<Block>,
}

impl UndoCutBuilder {
    pub fn new() -> Self {
        UndoCutBuilder {
            delete_blocks: vec![],
            restore_blocks: vec![],
        }
    }

    pub fn remove(&mut self, canvas: &mut Canvas, block_id: &BlockId) -> Result<Block, MoveError> {
        let block = canvas.remove_move_block(block_id)?;
        self.restore_blocks.push(block.clone());
        Ok(block)
    }

    pub fn create(&mut self, canvas: &mut Canvas, block: Block) {
        self.delete_blocks.push(block.id.clone());
        canvas.put_block(block)
    }

    fn build(self, canvas: &mut Canvas) -> UndoMove {
        UndoMove::cut(canvas, self.delete_blocks, self.restore_blocks)
    }
}

pub fn line_cut(
    canvas: &mut Canvas,
    block: &BlockId,
    orientation: Orientation,
    offset: u32,
) -> Result<(Cost, UndoMove), MoveError> {
    match orientation {
        Orientation::Horizontal => horizontal_cut(canvas, block, offset),
        Orientation::Vertical => vertical_cut(canvas, block, offset),
    }
}

pub fn vertical_cut(
    canvas: &mut Canvas,
    block_id: &BlockId,
    cut_offset_x: u32,
) -> Result<(Cost, UndoMove), MoveError> {
    let mut builder = UndoCutBuilder::new();
    let block = builder.remove(canvas, block_id)?;
    let cost = Cost::compute(MoveType::LineCut, block.size(), canvas.area);
    if !(block.r.bottom_left.x <= cut_offset_x && cut_offset_x < block.r.top_right.x) {
        return Err(MoveError::LogicError(format!(
            "Line number is out of the [{:?}]! Block is from {:?} to {:?}, point is at {:?}",
            block_id, block.r.bottom_left, block.r.top_right, cut_offset_x
        )));
    }

    match block.data {
        BlockData::Simple(_) => {
            let (left_r, right_r) = block.r.vertical_cut(cut_offset_x);
            builder.create(canvas, block.split_simple("0", left_r));
            builder.create(canvas, block.split_simple("1", right_r));
        }
        BlockData::Complex(bs) => {
            let mut left_blocks: Vec<SubBlock> = vec![];
            let mut right_blocks: Vec<SubBlock> = vec![];
            for child in bs {
                if child.r.bottom_left.x >= cut_offset_x {
                    right_blocks.push(child);
                    continue;
                }
                if child.r.top_right.x <= cut_offset_x {
                    left_blocks.push(child);
                    continue;
                }
                let (left_r, right_r) = child.r.vertical_cut(cut_offset_x);
                left_blocks.push(SubBlock::new(left_r, child.c));
                right_blocks.push(SubBlock::new(right_r, child.c));
            }

            let (left_r, right_r) = block.r.vertical_cut(cut_offset_x);
            builder.create(
                canvas,
                Block::new_complex(block_id.new_child("0"), left_r, left_blocks),
            );
            builder.create(
                canvas,
                Block::new_complex(block_id.new_child("1"), right_r, right_blocks),
            );
        }
    }
    Ok((cost, builder.build(canvas)))
}

pub fn horizontal_cut(
    canvas: &mut Canvas,
    block_id: &BlockId,
    cut_offset_y: u32,
) -> Result<(Cost, UndoMove), MoveError> {
    let mut builder = UndoCutBuilder::new();
    let block = builder.remove(canvas, block_id)?;
    let cost = Cost::compute(MoveType::LineCut, block.size(), canvas.area);
    if !(block.r.bottom_left.y <= cut_offset_y && cut_offset_y < block.r.top_right.y) {
        return Err(MoveError::LogicError(format!(
            "Col number is out of the [{:?}]! Block is from {:?} to {:?}, point is at {:?}",
            block_id, block.r.bottom_left, block.r.top_right, cut_offset_y
        )));
    }

    match block.data {
        BlockData::Simple(_) => {
            let (bottom_r, top_r) = block.r.horizontal_cut(cut_offset_y);
            builder.create(canvas, block.split_simple("0", bottom_r));
            builder.create(canvas, block.split_simple("1", top_r));
        }
        BlockData::Complex(bs) => {
            let mut bottom_blocks: Vec<SubBlock> = vec![];
            let mut top_blocks: Vec<SubBlock> = vec![];
            for child in bs {
                if child.r.bottom_left.y >= cut_offset_y {
                    top_blocks.push(child);
                    continue;
                }
                if child.r.top_right.y <= cut_offset_y {
                    bottom_blocks.push(child);
                    continue;
                }
                let (bottom_r, top_r) = child.r.horizontal_cut(cut_offset_y);
                bottom_blocks.push(SubBlock::new(bottom_r, child.c));
                top_blocks.push(SubBlock::new(top_r, child.c));
            }

            let (bottom_r, top_r) = block.r.horizontal_cut(cut_offset_y);
            builder.create(
                canvas,
                Block::new_complex(block_id.new_child("0"), bottom_r, bottom_blocks),
            );
            builder.create(
                canvas,
                Block::new_complex(block_id.new_child("1"), top_r, top_blocks),
            );
        }
    }
    Ok((cost, builder.build(canvas)))
}

pub fn point_cut(
    canvas: &mut Canvas,
    block_id: &BlockId,
    cut_x: u32,
    cut_y: u32,
) -> Result<(Cost, UndoMove), MoveError> {
    let cut_point = Point::new(cut_x, cut_y);
    let mut builder = UndoCutBuilder::new();
    let block = builder.remove(canvas, block_id)?;
    let cost = Cost::compute(MoveType::PointCut, block.size(), canvas.area);

    if !block.r.contains(cut_x, cut_y) {
        return Err(MoveError::LogicError(format!(
            "Point is out of [{}]! Block is from {:?} to {:?}, point is at {} {}!",
            block_id, block.r.bottom_left, block.r.top_right, cut_x, cut_y
        )));
    }

    let bs = match block.data {
        BlockData::Simple(_) => {
            let (bottom_left_bl, bottom_right_bl, top_right_bl, top_left_bl) =
                block.r.cross_cut(cut_x, cut_y);
            builder.create(canvas, block.split_simple("0", bottom_left_bl));
            builder.create(canvas, block.split_simple("1", bottom_right_bl));
            builder.create(canvas, block.split_simple("2", top_right_bl));
            builder.create(canvas, block.split_simple("3", top_left_bl));
            return Ok((cost, builder.build(canvas)));
        }
        BlockData::Complex(bs) => bs,
    };

    let mut bottom_left_blocks: Vec<SubBlock> = vec![];
    let mut bottom_right_blocks: Vec<SubBlock> = vec![];
    let mut top_right_blocks: Vec<SubBlock> = vec![];
    let mut top_left_blocks: Vec<SubBlock> = vec![];
    for child in bs {
        /*
         * __________________________
         * |        |       |       |
         * |   1    |   2   |   3   |
         * |________|_______|_______|
         * |        |       |       |
         * |   4    |   5   |  6    |
         * |________|_______|_______|
         * |        |       |       |
         * |   7    |   8   |   9   |
         * |________|_______|_______|
         */
        // Case 2
        if child.r.bottom_left.x >= cut_x && child.r.bottom_left.y >= cut_y {
            top_right_blocks.push(child);
            continue;
        }
        // Case 7
        if child.r.top_right.x <= cut_x && child.r.top_right.y <= cut_y {
            bottom_left_blocks.push(child);
            continue;
        }
        // Case 1
        if child.r.top_right.x <= cut_x && child.r.bottom_left.y >= cut_y {
            top_left_blocks.push(child);
            continue;
        }
        // Case 9
        if child.r.bottom_left.x >= cut_x && child.r.top_right.y <= cut_y {
            bottom_right_blocks.push(child);
            continue;
        }
        // Case 5
        if child.r.strictly_contains(cut_x, cut_y) {
            let (bl, br, tr, tl) = child.r.cross_cut(cut_x, cut_y);
            bottom_left_blocks.push(SubBlock::new(bl, child.c));
            bottom_right_blocks.push(SubBlock::new(br, child.c));
            top_right_blocks.push(SubBlock::new(tr, child.c));
            top_left_blocks.push(SubBlock::new(tl, child.c));
            continue;
        }

        // Case 2
        if child.r.bottom_left.x <= cut_x
            && cut_x <= child.r.top_right.x
            && cut_y <= child.r.bottom_left.y
        {
            top_left_blocks.push(SubBlock::new(
                Rect::new(child.r.bottom_left, Point::new(cut_x, child.r.top_right.y)),
                child.c,
            ));
            top_right_blocks.push(SubBlock::new(
                Rect::new(Point::new(cut_x, child.r.bottom_left.y), child.r.top_right),
                child.c,
            ));
            continue;
        }
        // Case 8
        if child.r.bottom_left.x <= cut_x
            && cut_x <= child.r.top_right.x
            && cut_y >= child.r.top_right.y
        {
            bottom_left_blocks.push(SubBlock::new(
                Rect::new(child.r.bottom_left, Point::new(cut_x, child.r.top_right.y)),
                child.c,
            ));
            bottom_right_blocks.push(SubBlock::new(
                Rect::new(Point::new(cut_x, child.r.bottom_left.y), child.r.top_right),
                child.c,
            ));
            continue;
        }
        // Case 4
        if child.r.bottom_left.y <= cut_y
            && cut_y <= child.r.top_right.y
            && cut_x <= child.r.bottom_left.x
        {
            bottom_right_blocks.push(SubBlock::new(
                Rect::new(child.r.bottom_left, Point::new(child.r.top_right.x, cut_y)),
                child.c,
            ));
            top_right_blocks.push(SubBlock::new(
                Rect::new(Point::new(child.r.bottom_left.x, cut_y), child.r.top_right),
                child.c,
            ));
            continue;
        }
        // Case 6
        if child.r.bottom_left.y <= cut_y
            && cut_y <= child.r.top_right.y
            && cut_x >= child.r.top_right.x
        {
            bottom_left_blocks.push(SubBlock::new(
                Rect::new(child.r.bottom_left, Point::new(child.r.top_right.x, cut_y)),
                child.c,
            ));
            top_left_blocks.push(SubBlock::new(
                Rect::new(Point::new(child.r.bottom_left.x, cut_y), child.r.top_right),
                child.c,
            ));
            continue;
        }
    }
    let bottom_left_block = Block::new_complex(
        block_id.new_child("0"),
        Rect::new(block.r.bottom_left, cut_point),
        bottom_left_blocks,
    );
    let bottom_right_block = Block::new_complex(
        block_id.new_child("1"),
        Rect::new(
            Point::new(cut_x, block.r.bottom_left.y),
            Point::new(block.r.top_right.x, cut_y),
        ),
        bottom_right_blocks,
    );
    let top_right_block = Block::new_complex(
        block_id.new_child("2"),
        Rect::new(cut_point, block.r.top_right),
        top_right_blocks,
    );
    let top_left_block = Block::new_complex(
        block_id.new_child("3"),
        Rect::new(
            Point::new(block.r.bottom_left.x, cut_y),
            Point::new(cut_x, block.r.top_right.y),
        ),
        top_left_blocks,
    );

    builder.create(canvas, bottom_left_block);
    builder.create(canvas, bottom_right_block);
    builder.create(canvas, top_right_block);
    builder.create(canvas, top_left_block);
    Ok((cost, builder.build(canvas)))
}
