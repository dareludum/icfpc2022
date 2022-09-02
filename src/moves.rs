use crate::{
    block::{Block, BlockId, Color, Rect, SimpleBlock},
    canvas::Canvas,
};

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

pub enum Move {
    LineCut(BlockId, Orientation, u32),
    PointCut(BlockId, u32, u32),
    Color(BlockId, Color),
    Swap(BlockId, BlockId),
    Merge(BlockId, BlockId),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cost(u32);

#[derive(Debug, Clone)]
struct MoveError(String);

impl Move {
    pub fn apply(&self, canvas: &mut Canvas) -> Result<Cost, MoveError> {
        match *self {
            Move::LineCut(ref block, orientation, offset) => {
                self.line_cut(canvas, block, orientation, offset)
            }
            Move::PointCut(ref block, x, y) => self.point_cut(canvas, block, x, y),
            Move::Color(ref block, c) => self.color(canvas, block, c),
            Move::Swap(ref block_a, ref block_b) => self.swap(canvas, block_a, block_b),
            Move::Merge(ref block_a, ref block_b) => self.merge(canvas, block_a, block_b),
        }
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
        Cost((self.base_cost() as f32 * (canvas_area as f32 / block_area as f32)).round() as u32)
    }

    fn color(
        &self,
        canvas: &mut Canvas,
        block_id: &BlockId,
        new_color: Color,
    ) -> Result<Cost, MoveError> {
        let canvas_area = canvas.area;
        let block = canvas.get_move_block_mut(block_id)?;
        let cost = self.compute_cost(block.size(), canvas_area);
        let (block_id, rect) = match block {
            // if the block is simple, change its color
            Block::Simple(ref mut simple) => {
                simple.c = new_color;
                return Ok(cost);
            }
            // if its complex, turn it into a simple block
            Block::Complex(ref mut complex) => (complex.id.clone(), complex.r.clone()),
        };

        *block = Block::Simple(SimpleBlock::new(block_id, rect, new_color));
        return Ok(cost);
    }

    fn line_cut(
        &self,
        canvas: &mut Canvas,
        block: &BlockId,
        orientation: Orientation,
        offset: u32,
    ) -> Result<Cost, MoveError> {
        match orientation {
            Orientation::Horizontal => self.horizontal_cut(canvas, block, offset),
            Orientation::Vertical => self.vertical_cut(canvas, block, offset),
        }
    }

    fn vertical_cut(
        &self,
        canvas: &mut Canvas,
        block_id: &BlockId,
        offset: u32,
    ) -> Result<Cost, MoveError> {
        let canvas_area = canvas.area;
        let block = canvas.remove_move_block(block_id)?;
        let cost = self.compute_cost(block.size(), canvas_area);
        todo!()
    }

    fn horizontal_cut(
        &self,
        canvas: &mut Canvas,
        block: &BlockId,
        offset: u32,
    ) -> Result<Cost, MoveError> {
        todo!()
    }

    fn point_cut(
        &self,
        canvas: &mut Canvas,
        block: &BlockId,
        offset_x: u32,
        offset_y: u32,
    ) -> Result<Cost, MoveError> {
        todo!()
    }

    fn swap(
        &self,
        canvas: &mut Canvas,
        block0: &BlockId,
        block1: &BlockId,
    ) -> Result<Cost, MoveError> {
        // assert!(block0.rect() == block1.rect());
        // std::mem::swap(block0, block1);
        // Move::Swap(block1.id().clone(), block0.id().clone())
        todo!()
    }

    fn merge(
        &self,
        canvas: &mut Canvas,
        block0: &BlockId,
        block1: &BlockId,
    ) -> Result<Cost, MoveError> {
        todo!()
    }
}

impl Canvas {
    fn get_move_block(&self, block_id: &BlockId) -> Result<&Block, MoveError> {
        match self.get_block(block_id) {
            Some(block) => Ok(block),
            None => Err(MoveError(format!("missing block: {}", block_id))),
        }
    }

    fn get_move_block_mut(&mut self, block_id: &BlockId) -> Result<&mut Block, MoveError> {
        match self.get_block_mut(block_id) {
            Some(block) => Ok(block),
            None => Err(MoveError(format!("missing block: {}", block_id))),
        }
    }

    fn remove_move_block(&mut self, block_id: &BlockId) -> Result<Block, MoveError> {
        match self.remove_block(block_id) {
            Some(block) => Ok(block),
            None => Err(MoveError(format!("missing block: {}", block_id))),
        }
    }
}
