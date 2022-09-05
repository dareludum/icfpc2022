use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap,
};

use crate::{
    block::BlockId,
    canvas::Canvas,
    color::Color,
    moves::{AppliedMove, Cost, Move, MoveType},
    painting::Painting,
    solvers::Processor,
};

pub struct Recolorv2;

impl Processor for Recolorv2 {
    fn name(&self) -> &str {
        "recolor"
    }

    fn process(
        &self,
        applied_moves: &mut Vec<AppliedMove>,
        canvas: &mut Canvas,
        painting: &Painting,
    ) {
        // step 1: find the optimal target coloring for all final blocks
        let mut best_colors: HashMap<BlockId, Color> =
            HashMap::with_capacity(canvas.blocks_count());
        for block in canvas.blocks_iter() {
            let pixels = painting.get_pixels(&block.r);
            const EPS: f32 = 0.2;
            const MAX_ITERATIONS: u32 = 1000;
            let color_options = [
                Color::gmedian(&pixels, EPS, MAX_ITERATIONS),
                Color::pmedian(&pixels, EPS, MAX_ITERATIONS),
            ];

            let best_color = color_options
                .iter()
                .min_by_key(|c| painting.calculate_score_rect(&block.r, **c) as i64)
                .unwrap();
            best_colors.insert(block.id.clone(), *best_color);
        }

        // step 2: reverse sort final blocks by cost of coloring
        let mut expensive_blocks: Vec<(BlockId, Cost)> = Vec::with_capacity(canvas.blocks_count());
        for block in canvas.blocks_iter() {
            expensive_blocks.push((
                block.id.clone(),
                canvas.compute_cost(MoveType::Color, block.area()),
            ));
        }
        expensive_blocks.sort_by_key(|(_, cost)| -(cost.0 as i64));

        // TODO: do not color blocks if coloring does not improve the result. we'll need this stuff at this point
        // let blocks_coloring_cost: HashMap<BlockId, Cost> = expensive_blocks
        //     .iter()
        //     .map(|(id, cost)| (id.clone(), *cost))
        //     .collect();

        // step 3: starting from the most expensive blocks to allocate, find time to color the block.
        //         The coloring time is identified using the ID of a parent block created that turn.
        //         start from the root block. if it is already scheduled for coloring, go to its child.
        //         Otherwise, schedule it for coloring our block.
        //         If all parents are taken, color at the end.
        let mut planned_creation_colors: HashMap<BlockId, Color> =
            HashMap::with_capacity(canvas.blocks_count());
        let mut additional_colors: Vec<(BlockId, Color)> = vec![];
        // for each block
        'block_loop: for (cur_block, _) in expensive_blocks {
            // reverse iterate over its parents
            let block_best_color = best_colors[&cur_block];
            for parent in cur_block.rev_parents() {
                // keep looking for a parent that does not yet have its coloring planned
                let unplanned_parent = match planned_creation_colors.entry(parent.clone()) {
                    Vacant(entry) => entry,
                    Occupied(_) => continue,
                };
                unplanned_parent.insert(block_best_color);
                continue 'block_loop;
            }
            // if not such parent is found, plan coloring at the end
            additional_colors.push((cur_block, block_best_color));
        }

        // step 4  Rebuild the coloring history.
        // step 4.1 make a color free move history
        let color_free_moves: Vec<_> = applied_moves
            .iter()
            .filter(|&mov| match mov.mov {
                Move::Color(..) => false,
                _ => true,
            })
            .collect();

        // step 4.2 go back to the initial canvas
        for am in applied_moves.iter().rev() {
            am.clone().undo(canvas);
        }

        // step 4.3, build the new moves by inserting coloring as color free moves are applied
        let mut new_moves: Vec<AppliedMove> = vec![];
        for free_move in color_free_moves {
            let applied_move = free_move.mov.clone().apply(canvas).unwrap();
            new_moves.push(applied_move.clone());
            for created_block in applied_move.created_blocks() {
                if let Some(color) = planned_creation_colors.get(&created_block) {
                    new_moves.push(Move::Color(created_block, *color).apply(canvas).unwrap());
                }
            }
        }
        for (block_id, color) in additional_colors {
            new_moves.push(Move::Color(block_id, color).apply(canvas).unwrap());
        }

        *applied_moves = new_moves;
    }
}
