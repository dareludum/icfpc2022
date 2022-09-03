use crate::block::BlockId;
use crate::block::{Point, Rect};
use crate::canvas::Canvas;
use crate::moves::{
    Block, ComplexBlock, Cost, Move, MoveError, Orientation, SimpleBlock, UndoMove,
};

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
        self.delete_blocks.push(block.get_id().clone());
        canvas.put_block(block)
    }

    fn build(self, canvas: &mut Canvas) -> UndoMove {
        UndoMove::cut(canvas, self.delete_blocks, self.restore_blocks)
    }
}

pub fn line_cut(
    mov: &Move,
    canvas: &mut Canvas,
    block: &BlockId,
    orientation: Orientation,
    offset: u32,
) -> Result<(Cost, UndoMove), MoveError> {
    match orientation {
        Orientation::Horizontal => horizontal_cut(mov, canvas, block, offset),
        Orientation::Vertical => vertical_cut(mov, canvas, block, offset),
    }
}

pub fn vertical_cut(
    mov: &Move,
    canvas: &mut Canvas,
    block_id: &BlockId,
    cut_offset_x: u32,
) -> Result<(Cost, UndoMove), MoveError> {
    let mut builder = UndoCutBuilder::new();
    let block = builder.remove(canvas, block_id)?;
    let cost = Cost::compute(mov, block.size(), canvas.area);
    if !(block.rect().bottom_left.x <= cut_offset_x && cut_offset_x < block.rect().top_right.x) {
        return Err(MoveError::LogicError(format!(
            "Line number is out of the [{:?}]! Block is from {:?} to {:?}, point is at {:?}",
            block_id,
            block.rect().bottom_left,
            block.rect().top_right,
            cut_offset_x
        )));
    }

    match block {
        Block::Simple(simple) => {
            let (left_r, right_r) = simple.r.vertical_cut(cut_offset_x);
            builder.create(canvas, simple.split("0", left_r).into());
            builder.create(canvas, simple.split("1", right_r).into());
        }
        Block::Complex(complex) => {
            let mut left_blocks: Vec<SimpleBlock> = vec![];
            let mut right_blocks: Vec<SimpleBlock> = vec![];
            for child in complex.bs {
                if child.r.bottom_left.x >= cut_offset_x {
                    right_blocks.push(child);
                    continue;
                }
                if child.r.top_right.x <= cut_offset_x {
                    left_blocks.push(child);
                    continue;
                }
                let (left_r, right_r) = child.r.vertical_cut(cut_offset_x);
                left_blocks.push(child.complex_split("left", left_r));
                right_blocks.push(child.complex_split("right", right_r));
            }

            let (left_r, right_r) = complex.r.vertical_cut(cut_offset_x);
            builder.create(
                canvas,
                ComplexBlock::new(block_id.new_child("0"), left_r, left_blocks).into(),
            );
            builder.create(
                canvas,
                ComplexBlock::new(block_id.new_child("1"), right_r, right_blocks).into(),
            );
        }
    }
    Ok((cost, builder.build(canvas)))
}

pub fn horizontal_cut(
    mov: &Move,
    canvas: &mut Canvas,
    block_id: &BlockId,
    cut_offset_y: u32,
) -> Result<(Cost, UndoMove), MoveError> {
    let mut builder = UndoCutBuilder::new();
    let block = builder.remove(canvas, block_id)?;
    let cost = Cost::compute(mov, block.size(), canvas.area);
    if !(block.rect().bottom_left.y <= cut_offset_y && cut_offset_y < block.rect().top_right.y) {
        return Err(MoveError::LogicError(format!(
            "Col number is out of the [{:?}]! Block is from {:?} to {:?}, point is at {:?}",
            block_id,
            block.rect().bottom_left,
            block.rect().top_right,
            cut_offset_y
        )));
    }

    match block {
        Block::Simple(simple) => {
            let (bottom_r, top_r) = simple.r.horizontal_cut(cut_offset_y);
            builder.create(canvas, simple.split("0", bottom_r).into());
            builder.create(canvas, simple.split("1", top_r).into());
        }
        Block::Complex(complex) => {
            let mut bottom_blocks: Vec<SimpleBlock> = vec![];
            let mut top_blocks: Vec<SimpleBlock> = vec![];
            for child in complex.bs {
                if child.r.bottom_left.y >= cut_offset_y {
                    top_blocks.push(child);
                    continue;
                }
                if child.r.top_right.y <= cut_offset_y {
                    bottom_blocks.push(child);
                    continue;
                }
                let (bottom_r, top_r) = child.r.horizontal_cut(cut_offset_y);
                bottom_blocks.push(child.complex_split("bottom", bottom_r));
                top_blocks.push(child.complex_split("top", top_r));
            }

            let (bottom_r, top_r) = complex.r.horizontal_cut(cut_offset_y);
            builder.create(
                canvas,
                ComplexBlock::new(block_id.new_child("0"), bottom_r, bottom_blocks).into(),
            );
            builder.create(
                canvas,
                ComplexBlock::new(block_id.new_child("1"), top_r, top_blocks).into(),
            );
        }
    }
    Ok((cost, builder.build(canvas)))
}

pub fn point_cut(
    mov: &Move,
    canvas: &mut Canvas,
    block_id: &BlockId,
    cut_x: u32,
    cut_y: u32,
) -> Result<(Cost, UndoMove), MoveError> {
    let cut_point = Point::new(cut_x, cut_y);
    let mut builder = UndoCutBuilder::new();
    let block = builder.remove(canvas, block_id)?;
    let cost = Cost::compute(mov, block.size(), canvas.area);

    if !block.rect().contains(cut_x, cut_y) {
        return Err(MoveError::LogicError(format!(
            "Point is out of [{}]! Block is from {:?} to {:?}, point is at {} {}!",
            block_id,
            block.rect().bottom_left,
            block.rect().top_right,
            cut_x,
            cut_y
        )));
    }

    let complex_block = match block {
        Block::Simple(simple) => {
            let (bottom_left_bl, bottom_right_bl, top_right_bl, top_left_bl) =
                simple.r.cross_cut(cut_x, cut_y);
            builder.create(canvas, simple.split("0", bottom_left_bl).into());
            builder.create(canvas, simple.split("1", bottom_right_bl).into());
            builder.create(canvas, simple.split("2", top_right_bl).into());
            builder.create(canvas, simple.split("3", top_left_bl).into());
            return Ok((cost, builder.build(canvas)));
        }
        Block::Complex(complex) => complex,
    };

    let mut bottom_left_blocks: Vec<SimpleBlock> = vec![];
    let mut bottom_right_blocks: Vec<SimpleBlock> = vec![];
    let mut top_right_blocks: Vec<SimpleBlock> = vec![];
    let mut top_left_blocks: Vec<SimpleBlock> = vec![];
    for child in complex_block.bs {
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
        if child.r.contains(cut_x, cut_y) {
            let (bl, br, tr, tl) = child.r.cross_cut(cut_x, cut_y);
            bottom_left_blocks.push(child.complex_split("bl_child", bl));
            bottom_right_blocks.push(child.complex_split("br_child", br));
            top_right_blocks.push(child.complex_split("tr_child", tr));
            top_left_blocks.push(child.complex_split("tl_child", tl));
            continue;
        }

        // Case 2
        if child.r.bottom_left.x <= cut_x
            && cut_x <= child.r.top_right.x
            && cut_y < child.r.bottom_left.y
        {
            top_left_blocks.push(SimpleBlock::new(
                "case2_tl_child".into(),
                Rect::new(child.r.bottom_left, Point::new(cut_x, child.r.top_right.y)),
                child.c,
            ));
            top_right_blocks.push(SimpleBlock::new(
                "case2_tr_child".into(),
                Rect::new(Point::new(cut_x, child.r.bottom_left.y), child.r.top_right),
                child.c,
            ));
            continue;
        }
        // Case 8
        if child.r.bottom_left.x <= cut_x
            && cut_x <= child.r.top_right.x
            && cut_y > child.r.top_right.y
        {
            bottom_left_blocks.push(SimpleBlock::new(
                "case8_bl_child".into(),
                Rect::new(child.r.bottom_left, Point::new(cut_x, child.r.top_right.y)),
                child.c,
            ));
            bottom_right_blocks.push(SimpleBlock::new(
                "case8_br_child".into(),
                Rect::new(Point::new(cut_x, child.r.bottom_left.y), child.r.top_right),
                child.c,
            ));
            continue;
        }
        // Case 4
        if child.r.bottom_left.y <= cut_y
            && cut_y <= child.r.top_right.y
            && cut_x < child.r.bottom_left.x
        {
            bottom_right_blocks.push(SimpleBlock::new(
                "case4_br_child".into(),
                Rect::new(child.r.bottom_left, Point::new(child.r.top_right.x, cut_y)),
                child.c,
            ));
            top_right_blocks.push(SimpleBlock::new(
                "case4_tr_child".into(),
                Rect::new(Point::new(child.r.bottom_left.x, cut_y), child.r.top_right),
                child.c,
            ));
            continue;
        }
        // Case 6
        if child.r.bottom_left.y <= cut_y
            && cut_y <= child.r.top_right.y
            && cut_x > child.r.top_right.x
        {
            bottom_left_blocks.push(SimpleBlock::new(
                "case6_bl_child".into(),
                Rect::new(child.r.bottom_left, Point::new(child.r.top_right.x, cut_y)),
                child.c,
            ));
            top_left_blocks.push(SimpleBlock::new(
                "case6_br_child".into(),
                Rect::new(Point::new(child.r.bottom_left.x, cut_y), child.r.top_right),
                child.c,
            ));
            continue;
        }
    }
    let bottom_left_block = ComplexBlock::new(
        block_id.new_child("0"),
        Rect::new(complex_block.r.bottom_left, cut_point),
        bottom_left_blocks,
    );
    let bottom_right_block = ComplexBlock::new(
        block_id.new_child("1"),
        Rect::new(
            Point::new(cut_x, complex_block.r.bottom_left.y),
            Point::new(complex_block.r.top_right.x, cut_y),
        ),
        bottom_right_blocks,
    );
    let top_right_block = ComplexBlock::new(
        block_id.new_child("2"),
        Rect::new(cut_point, complex_block.r.top_right),
        top_right_blocks,
    );
    let top_left_block = ComplexBlock::new(
        block_id.new_child("3"),
        Rect::new(
            Point::new(complex_block.r.bottom_left.x, cut_y),
            Point::new(cut_x, complex_block.r.top_right.y),
        ),
        top_left_blocks,
    );

    builder.create(canvas, bottom_left_block.into());
    builder.create(canvas, bottom_right_block.into());
    builder.create(canvas, top_right_block.into());
    builder.create(canvas, top_left_block.into());
    Ok((cost, builder.build(canvas)))
}
