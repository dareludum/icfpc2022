use serde::{Deserialize, Serialize};
use smartstring::{LazyCompact, SmartString};

#[derive(Deserialize, Debug)]
pub struct BlockDto {
    #[serde(rename(deserialize = "blockId"))]
    pub block_id: SmartString<LazyCompact>,
    #[serde(rename(deserialize = "bottomLeft"))]
    pub bottom_left: [u32; 2],
    #[serde(rename(deserialize = "topRight"))]
    pub top_right: [u32; 2],
    pub color: Option<[u8; 4]>,
    #[serde(rename(deserialize = "pngBottomLeftPoint"))]
    pub png_bottom_left: Option<[u32; 2]>,
}

#[derive(Deserialize, Debug)]
pub struct CanvasDto {
    pub width: u32,
    pub height: u32,
    #[serde(rename(deserialize = "sourcePngPNG"))]
    pub source_png: Option<String>,
    pub blocks: Vec<BlockDto>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SolvedSolutionDto {
    pub solver_name: String,
    pub total_score: u64,
    pub score: u64,
    pub solution_cost: u64,
}

impl SolvedSolutionDto {
    pub fn summarize(&self) -> String {
        format!(
            "total {} = (delta {} + moves {})",
            self.total_score, self.score, self.solution_cost
        )
    }

    pub fn not_solved() -> Self {
        SolvedSolutionDto {
            solver_name: "err_not_solved".to_string(),
            total_score: u64::MAX,
            score: u64::MAX,
            solution_cost: u64::MAX,
        }
    }
}
