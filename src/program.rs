use crate::{block::Color, moves::Move};

fn to_isl(mov: Move) -> String {
    match mov {
        Move::LineCut(bid, orientation, lnum) => {
            format!("cut [ {bid} ] [{orientation}] [ {lnum} ]")
        }
        Move::PointCut(bid, x, y) => format!("cut [ {bid} ] [ {x} , {y} ]"),
        Move::Color(bid, Color { r, g, b, a }) => {
            format!("color [ {bid} ] [ {r} , {g} , {b} , {a} ]")
        }
        Move::Swap(bid1, bid2) => format!("swap [ {bid1} ] [ {bid2} ]"),
        Move::Merge(bid1, bid2) => format!("merge [ {bid1} ] [ {bid2} ]"),
    }
}

pub fn generate_isl(moves: Vec<Move>) -> String {
    let mut isl = String::new();

    for mov in moves {
        let mov_isl = to_isl(mov);
        isl += &format!("{mov_isl}\n");
    }

    isl
}
