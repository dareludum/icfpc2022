use derive_more::{Add, AddAssign};

use std::fmt::Display;

use crate::{
    block::{Block, BlockId, ComplexBlock, Rect, SimpleBlock},
    canvas::Canvas,
    color::Color,
};

mod cut;

pub use cut::*;

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
pub enum MoveError {
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
