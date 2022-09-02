use std::{fs, path::Path};

use crate::{block::Color, moves::Move};

fn to_isl(mov: &Move) -> String {
    match mov {
        Move::LineCut(bid, orientation, lnum) => {
            format!("cut[{bid}][{orientation}][{lnum}]")
        }
        Move::PointCut(bid, x, y) => format!("cut[{bid}][{x},{y}]"),
        Move::Color(bid, Color { r, g, b, a }) => {
            format!("color[{bid}][{r},{g},{b},{a}]")
        }
        Move::Swap(bid1, bid2) => format!("swap[{bid1}][{bid2}]"),
        Move::Merge(bid1, bid2) => format!("merge[{bid1}][{bid2}]"),
    }
}

pub fn generate_isl(moves: &Vec<Move>) -> String {
    let mut isl = String::new();

    for mov in moves {
        let mov_isl = to_isl(mov);
        isl += &format!("{mov_isl}\n");
    }

    isl
}

pub fn write_to_file(path: &Path, moves: &Vec<Move>) -> std::io::Result<()> {
    fs::write(path, generate_isl(moves))
}

#[cfg(test)]
mod tests {
    use crate::moves::Orientation;

    use super::*;

    #[test]
    fn generate_isl_test() {
        let moves = vec![
            Move::LineCut("0".to_string(), Orientation::Vertical, 12),
            Move::PointCut("1".to_string(), 44, 44),
            Move::LineCut("2".to_string(), Orientation::Horizontal, 1),
            Move::Color(
                "1.1".to_string(),
                Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 0,
                },
            ),
            Move::Merge("1".to_string(), "2".to_string()),
            Move::Swap("3.0.0".to_string(), "0.0.1".to_string()),
        ];

        let expected = "\
cut[0][x][12]
cut[1][44,44]
cut[2][y][1]
color[1.1][255,255,255,0]
merge[1][2]
swap[3.0.0][0.0.1]
";

        assert_eq!(generate_isl(&moves), expected);
    }
}
