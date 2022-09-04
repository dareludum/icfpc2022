use std::fmt::Write as _;
use std::{fs, path::Path};

use crate::moves::Move;

pub fn to_isl(mov: &Move) -> String {
    match mov {
        Move::LineCut(bid, orientation, lnum) => {
            format!("cut[{bid}][{orientation}][{lnum}]")
        }
        Move::PointCut(bid, x, y) => format!("cut[{bid}][{x},{y}]"),
        Move::Color(bid, color) => {
            format!(
                "color[{bid}][{},{},{},{}]",
                color.r(),
                color.g(),
                color.b(),
                color.a()
            )
        }
        Move::Swap(bid1, bid2) => format!("swap[{bid1}][{bid2}]"),
        Move::Merge(bid1, bid2) => format!("merge[{bid1}][{bid2}]"),
    }
}

pub fn generate_isl(moves: &Vec<Move>) -> String {
    let mut isl = String::new();

    for mov in moves {
        let mov_isl = to_isl(mov);
        writeln!(isl, "{mov_isl}").expect("can't write to isl string");
    }

    isl
}

pub fn write_to_file(path: &Path, moves: &Vec<Move>) -> std::io::Result<()> {
    fs::write(path, generate_isl(moves))
}

#[cfg(test)]
mod tests {
    use crate::{color::Color, moves::Orientation};

    use super::*;

    #[test]
    fn generate_isl_test() {
        let moves = vec![
            Move::LineCut("0".into(), Orientation::Vertical, 12),
            Move::PointCut("1".into(), 44, 44),
            Move::LineCut("2".into(), Orientation::Horizontal, 1),
            Move::Color("1.1".into(), Color::new(255, 255, 255, 0)),
            Move::Merge("1".into(), "2".into()),
            Move::Swap("3.0.0".into(), "0.0.1".into()),
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
