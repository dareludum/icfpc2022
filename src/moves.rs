use derive_more::{Add, AddAssign};

use std::fmt::Display;

use crate::{
    block::{Block, BlockId, ComplexBlock, Point, Rect, SimpleBlock},
    canvas::Canvas,
    color::Color,
};

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub enum Move {
    LineCut(BlockId, Orientation, u32),
    PointCut(BlockId, u32, u32),
    Color(BlockId, Color),
    Swap(BlockId, BlockId),
    Merge(BlockId, BlockId),
}

#[derive(Debug, Clone)]
pub enum UndoMove {
    Cut {
        delete_block_ids: Vec<BlockId>,
        restore_blocks: Vec<Block>,
    },
    SimpleColor {
        block_id: BlockId,
        prev_color: Color,
    },
    ComplexColor {
        old_block: Block,
    },
    Swap {
        a_id: BlockId,
        b_id: BlockId,
    },
    Merge {
        merged_block_id: BlockId,
        initial_a: Block,
        initial_b: Block,
    },
}

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

    // pub fn add_original_block(&mut self, block: Block) {
    //     self.restore_blocks.push(block);
    // }

    fn build(self) -> UndoMove {
        UndoMove::Cut {
            delete_block_ids: self.delete_blocks,
            restore_blocks: self.restore_blocks,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Add, AddAssign)]
pub struct Cost(pub u64);

impl Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Orientation::Horizontal => write!(f, "y"),
            Orientation::Vertical => write!(f, "x"),
        }
    }
}

#[derive(Debug, Clone)]
enum MoveError {
    LogicError(String),
    InvalidInput(String),
}

impl Move {
    pub fn apply(&self, canvas: &mut Canvas) -> Option<(Cost, UndoMove)> {
        let res: Result<(Cost, UndoMove), MoveError> = match *self {
            Move::LineCut(ref block, orientation, offset) => {
                self.line_cut(canvas, block, orientation, offset)
            }
            Move::PointCut(ref block, x, y) => self.point_cut(canvas, block, x, y),
            Move::Color(ref block, c) => self.color(canvas, block, c),
            Move::Swap(ref block_a, ref block_b) => self.swap(canvas, block_a, block_b),
            Move::Merge(ref block_a, ref block_b) => self.merge(canvas, block_a, block_b),
        };
        res.ok()
    }

    fn base_cost(&self) -> u32 {
        match self {
            Move::LineCut(..) => 7,
            Move::PointCut(..) => 10,
            Move::Color(..) => 5,
            Move::Swap(..) => 3,
            Move::Merge(..) => 1,
        }
    }

    fn compute_cost(&self, block_area: u32, canvas_area: u32) -> Cost {
        Cost((self.base_cost() as f64 * (canvas_area as f64 / block_area as f64)).round() as u64)
    }

    fn color(
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

    fn line_cut(
        &self,
        canvas: &mut Canvas,
        block: &BlockId,
        orientation: Orientation,
        offset: u32,
    ) -> Result<(Cost, UndoMove), MoveError> {
        match orientation {
            Orientation::Horizontal => self.horizontal_cut(canvas, block, offset),
            Orientation::Vertical => self.vertical_cut(canvas, block, offset),
        }
    }

    fn vertical_cut(
        &self,
        canvas: &mut Canvas,
        block_id: &BlockId,
        cut_offset_x: u32,
    ) -> Result<(Cost, UndoMove), MoveError> {
        let mut builder = UndoCutBuilder::new();
        let block = builder.remove(canvas, block_id)?;
        let cost = self.compute_cost(block.size(), canvas.area);
        if !(block.rect().bottom_left.x <= cut_offset_x && cut_offset_x < block.rect().top_right.x)
        {
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
                builder.create(canvas, simple.split(0, left_r).into());
                builder.create(canvas, simple.split(1, right_r).into());
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
                    ComplexBlock::new(block_id.to_owned() + ".0", left_r, left_blocks).into(),
                );
                builder.create(
                    canvas,
                    ComplexBlock::new(block_id.to_owned() + ".1", right_r, right_blocks).into(),
                );
            }
        }
        Ok((cost, builder.build()))
    }

    fn horizontal_cut(
        &self,
        canvas: &mut Canvas,
        block_id: &BlockId,
        cut_offset_y: u32,
    ) -> Result<(Cost, UndoMove), MoveError> {
        let mut builder = UndoCutBuilder::new();
        let block = builder.remove(canvas, block_id)?;
        let cost = self.compute_cost(block.size(), canvas.area);
        if !(block.rect().bottom_left.y <= cut_offset_y && cut_offset_y < block.rect().top_right.y)
        {
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
                builder.create(canvas, simple.split(0, bottom_r).into());
                builder.create(canvas, simple.split(1, top_r).into());
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
                    ComplexBlock::new(block_id.to_owned() + ".0", bottom_r, bottom_blocks).into(),
                );
                builder.create(
                    canvas,
                    ComplexBlock::new(block_id.to_owned() + ".1", top_r, top_blocks).into(),
                );
            }
        }
        Ok((cost, builder.build()))
    }

    fn point_cut(
        &self,
        canvas: &mut Canvas,
        block_id: &BlockId,
        cut_x: u32,
        cut_y: u32,
    ) -> Result<(Cost, UndoMove), MoveError> {
        let cut_point = Point::new(cut_x, cut_y);
        let mut builder = UndoCutBuilder::new();
        let block = builder.remove(canvas, block_id)?;
        let cost = self.compute_cost(block.size(), canvas.area);

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
                builder.create(canvas, simple.split(0, bottom_left_bl).into());
                builder.create(canvas, simple.split(1, bottom_right_bl).into());
                builder.create(canvas, simple.split(2, top_right_bl).into());
                builder.create(canvas, simple.split(3, top_left_bl).into());
                return Ok((cost, builder.build()));
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
            block_id.to_owned() + ".0",
            Rect::new(complex_block.r.bottom_left, cut_point),
            bottom_left_blocks,
        );
        let bottom_right_block = ComplexBlock::new(
            block_id.to_owned() + ".1",
            Rect::new(
                Point::new(cut_x, complex_block.r.bottom_left.y),
                Point::new(complex_block.r.top_right.x, cut_y),
            ),
            bottom_right_blocks,
        );
        let top_right_block = ComplexBlock::new(
            block_id.to_owned() + ".2",
            Rect::new(cut_point, complex_block.r.top_right),
            top_right_blocks,
        );
        let top_left_block = ComplexBlock::new(
            block_id.to_owned() + ".3",
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
        Ok((cost, builder.build()))
    }

    fn swap(
        &self,
        canvas: &mut Canvas,
        block_a_id: &BlockId,
        block_b_id: &BlockId,
    ) -> Result<(Cost, UndoMove), MoveError> {
        let mut block_a = canvas.remove_move_block(block_a_id)?;
        let mut block_b = canvas.remove_move_block(block_b_id)?;

        let cost = self.compute_cost(block_a.size(), canvas.area);

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
            UndoMove::Swap {
                a_id: block_a_id.clone(),
                b_id: block_b_id.clone(),
            },
        ))
    }

    fn merge(
        &self,
        canvas: &mut Canvas,
        block_a_id: &BlockId,
        block_b_id: &BlockId,
    ) -> Result<(Cost, UndoMove), MoveError> {
        let block_a = canvas.remove_move_block(block_a_id)?;
        let block_b = canvas.remove_move_block(block_b_id)?;
        let cost = self.compute_cost(std::cmp::max(block_a.size(), block_b.size()), canvas.area);
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

impl Canvas {
    fn get_move_block_mut(&mut self, block_id: &BlockId) -> Result<&mut Block, MoveError> {
        match self.get_block_mut(block_id) {
            Some(block) => Ok(block),
            None => Err(MoveError::LogicError(format!(
                "missing block: {}",
                block_id
            ))),
        }
    }

    fn remove_move_block(&mut self, block_id: &BlockId) -> Result<Block, MoveError> {
        match self.remove_block(block_id) {
            Some(block) => Ok(block),
            None => Err(MoveError::LogicError(format!(
                "missing block: {}",
                block_id
            ))),
        }
    }
}

impl UndoMove {
    pub fn apply(self, canvas: &mut Canvas) {
        match self {
            UndoMove::Cut {
                delete_block_ids: delete_blocks,
                restore_blocks,
            } => {
                for b in delete_blocks {
                    canvas.remove_block(&b);
                }
                for b in restore_blocks {
                    canvas.put_block(b);
                }
            }
            UndoMove::SimpleColor {
                block_id: block,
                prev_color,
            } => {
                let block = canvas.get_block_mut(&block).unwrap();
                if let Block::Simple(b) = block {
                    b.c = prev_color;
                } else {
                    panic!("Invalid block")
                }
            }
            UndoMove::ComplexColor { .. } => todo!(),
            UndoMove::Swap { a_id, b_id } => {
                Move::Swap(a_id, b_id).apply(canvas);
            }
            UndoMove::Merge { .. } => todo!(),
        }
    }
}
