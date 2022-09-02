use crate::block::{Block, BlockId, Color, Rect};

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

impl Move {
    pub fn base_cost(&self) -> u32 {
        match self {
            Move::LineCut(_, _, _) => 7,
            Move::PointCut(_, _, _) => 10,
            Move::Color(_, _) => 5,
            Move::Swap(_, _) => 3,
            Move::Merge(_, _) => 1,
        }
    }

    pub fn line_cut(block: &mut Block, orientation: Orientation, offset: u32) -> Self {
        todo!()
        // let id = block.id().clone();
        // match block {
        //     Block::Simple(id, rect, color) => {
        //         let (b0, b1) = match orientation {
        //             Orientation::Horizontal => {
        //                 let r0 = Rect::new(rect.x, rect.y, rect.w, offset);
        //                 let r1 = Rect::new(rect.x, rect.y, rect.w, rect.h - offset);
        //                 let mut id0 = id.clone();
        //                 id0.push(0);
        //                 let mut id1 = id.clone();
        //                 id1.push(1);
        //                 (
        //                     Block::Simple(id0, r0, *color),
        //                     Block::Simple(id1, r1, *color),
        //                 )
        //             }
        //             Orientation::Vertical => {
        //                 let r0 = Rect::new(rect.x, rect.y, offset, rect.h);
        //                 let r1 = Rect::new(rect.x, rect.y, rect.w - offset, rect.h);
        //                 let mut id0 = id.clone();
        //                 id0.push(0);
        //                 let mut id1 = id.clone();
        //                 id1.push(1);
        //                 (
        //                     Block::Simple(id0, r0, *color),
        //                     Block::Simple(id1, r1, *color),
        //                 )
        //             }
        //         };
        //         *block = Block::ComplexBlock(id.clone(), rect.clone(), vec![b0, b1]);
        //     }
        //     Block::ComplexBlock(_, _, _) => panic!("Invalid block"),
        // }
        // Move::LineCut(id, orientation, offset)
    }

    pub fn point_cut(block: &mut Block, offset_x: u32, offset_y: u32) -> Self {
        todo!()
        // let id = block.id().clone();
        // match block {
        //     Block::Simple(id, rect, color) => {
        //         let r0 = Rect::new(rect.x, rect.y, offset_x, offset_y);
        //         let r1 = Rect::new(offset_x, rect.y, rect.w - offset_x, offset_y);
        //         let r2 = Rect::new(offset_x, offset_y, rect.w - offset_x, rect.h - offset_y);
        //         let r3 = Rect::new(rect.x, offset_y, offset_x, rect.h - offset_y);
        //         let mut id0 = id.clone();
        //         id0.push(0);
        //         let mut id1 = id.clone();
        //         id1.push(1);
        //         let mut id2 = id.clone();
        //         id1.push(2);
        //         let mut id3 = id.clone();
        //         id1.push(3);
        //         let b0 = Block::Simple(id0, r0, *color);
        //         let b1 = Block::Simple(id1, r1, *color);
        //         let b2 = Block::Simple(id2, r2, *color);
        //         let b3 = Block::Simple(id3, r3, *color);
        //         *block = Block::ComplexBlock(id.clone(), rect.clone(), vec![b0, b1, b2, b3]);
        //     }
        //     Block::ComplexBlock(_, _, _) => panic!("Invalid block"),
        // }
        // Move::PointCut(id, offset_x, offset_y)
    }

    pub fn color(block: &mut Block, new_color: Color) -> Self {
        todo!()
        // let id = block.id().clone();
        // match block {
        //     Block::Simple(_, _, color) => {
        //         *color = new_color;
        //     }
        //     Block::ComplexBlock(_, _, _) => panic!("Invalid block"),
        // }
        // Move::Color(id, new_color)
    }

    pub fn swap(block0: &mut Block, block1: &mut Block) -> Self {
        assert!(block0.rect() == block1.rect());
        std::mem::swap(block0, block1);
        Move::Swap(block1.id().clone(), block0.id().clone())
    }

    pub fn merge(block0: Block, block1: Block) -> (Block, Self) {
        todo!()
        // let r0 = block0.rect();
        // let r1 = block1.rect();
        // let vertically_adjacent = r0.w == r1.w && ((r0.y + r0.h == r1.y) || (r0.y == r1.y + r1.h));
        // let horizontally_adjacent = r0.h == r1.h && ((r0.x + r0.w == r1.x) || (r0.x == r1.x + r1.w));
        // assert!(vertically_adjacent || horizontally_adjacent);
        // let id0 = block0.id().clone();
        // let id1 = block1.id().clone();
        // let rect = if vertically_adjacent {
        //     Rect::new(r0.x, r0.y.min(r1.y), r0.w, r0.h + r1.h)
        // } else {
        //     Rect::new(r0.x.min(r1.x), r1.y, r0.w + r1.w, r0.h)
        // };
        // let new_block = Block::ComplexBlock(BlockId::new(), rect, vec![block0, block1]);
        // (new_block, Move::Merge(id0, id1))
    }
}
