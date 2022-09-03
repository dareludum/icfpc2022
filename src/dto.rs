use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};

#[derive(Deserialize)]
pub struct BlockDto {
    #[serde(rename(deserialize = "blockId"))]
    pub block_id: SmartString<LazyCompact>,
    #[serde(rename(deserialize = "bottomLeft"))]
    pub bottom_left: [u32; 2],
    #[serde(rename(deserialize = "topRight"))]
    pub top_right: [u32; 2],
    pub color: [u8; 4],
}

#[derive(Deserialize)]
pub struct CanvasDto {
    pub width: u32,
    pub height: u32,
    pub blocks: Vec<BlockDto>,
}

#[derive(Serialize, Deserialize)]
pub struct SolvedSolutionDto {
    pub solver_name: String,
    pub total_score: u64,
    pub score: u64,
    pub solution_cost: u64,
}