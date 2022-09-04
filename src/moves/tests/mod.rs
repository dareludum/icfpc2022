use super::*;
use crate::block::*;

///  0,32             16,32    24,32    32,32
///   +-----------------+--------+--------+
///   |                 |        1        |
///   |                 |       /|\       |
///   |                 | 0.1.3  | 0.1.2  |  <=== these two blocks are children of complex block 1
///   |                 |        |        |
///   |                 |16,16   |24,16   |
///   |       0.0       +--------+--------+ 32,16
///   |                 |        |        |
///   |                 |        |        |
///   |                 | 0.1.0  | 0.1.1  |
///   |                 |        |        |
///   |                 |        |        |
///   +-----------------+--------+--------+
/// 0,0                16,0     24,0     32,0
fn make_complicated_canvas() -> Canvas {
    let bg = Color::new(255, 255, 255, 255);
    let mut blocks: Vec<Block> = vec![];
    blocks.push(Block::new_simple(
        "0.0".into(),
        Rect::from_coords([0, 0, 16, 32]),
        bg,
    ));
    blocks.push(Block::new_simple(
        "0.1.0".into(),
        Rect::from_coords([16, 0, 24, 16]),
        bg,
    ));
    blocks.push(Block::new_simple(
        "0.1.1".into(),
        Rect::from_coords([24, 0, 32, 16]),
        bg,
    ));
    blocks.push(Block::new_complex(
        "1".into(),
        Rect::from_coords([16, 16, 32, 32]),
        vec![
            SimpleBlock::new("0.1.2".into(), Rect::from_coords([24, 16, 32, 32]), bg),
            SimpleBlock::new("0.1.3".into(), Rect::from_coords([16, 16, 24, 32]), bg),
        ],
    ));
    // this is a 3rd generation canvas, as 3 moves were applied
    return Canvas::from_blocks(32, 32, 2, 3, blocks.into_iter());
}

#[test]
fn test_reproduce_complicated() -> Result<(), MoveError> {
    let mut canvas = Canvas::new(32, 32);
    Move::LineCut("0".into(), Orientation::Vertical, 16).checked_apply(&mut canvas)?;
    Move::PointCut("0.1".into(), 24, 16).checked_apply(&mut canvas)?;
    Move::Merge("0.1.2".into(), "0.1.3".into()).checked_apply(&mut canvas)?;
    let ref_canvas = make_complicated_canvas();
    assert_eq!(&canvas, &ref_canvas);
    Ok(())
}

///  0,32         16,32         32,32
///   +-------------+-------------+
///   |             |             |
///   |     3.3     |     3.2     |
///   |             |16,16        |
///   |-------------+-------------+
///   |             |             |
///   |     3.0     |     3.1     |
///   |             |             |
///   +-------------+-------------+
/// 0,0           16,0          32,0
///
/// This setup can be made by cutting a cross in the middle,
/// merging it all back together, and then splitting again.
fn make_cross_canvas() -> Canvas {
    let bg = Color::new(255, 255, 255, 255);
    let mut blocks: Vec<Block> = vec![];
    let bl = Rect::from_coords([0, 0, 16, 16]);
    let br = Rect::from_coords([16, 0, 32, 16]);
    let tr = Rect::from_coords([16, 16, 32, 32]);
    let tl = Rect::from_coords([0, 16, 16, 32]);
    blocks.push(
        Block::new_complex(
            "3.0".into(),
            bl,
            vec![SimpleBlock::new("0.0".into(), bl, bg)],
        )
        .into(),
    );
    blocks.push(
        Block::new_complex(
            "3.1".into(),
            br,
            vec![SimpleBlock::new("0.1".into(), br, bg)],
        )
        .into(),
    );
    blocks.push(
        Block::new_complex(
            "3.2".into(),
            tr,
            vec![SimpleBlock::new("0.2".into(), tr, bg)],
        )
        .into(),
    );
    blocks.push(
        Block::new_complex(
            "3.3".into(),
            tl,
            vec![SimpleBlock::new("0.3".into(), tl, bg)],
        )
        .into(),
    );
    // this is a 3rd generation canvas, as 3 moves were applied
    return Canvas::from_blocks(32, 32, 4, 5, blocks.into_iter());
}

#[test]
fn test_reproduce_cross() -> Result<(), MoveError> {
    let mut canvas = Canvas::new(32, 32);
    Move::PointCut("0".into(), 16, 16).checked_apply(&mut canvas)?;
    // create lower band 1
    Move::Merge("0.0".into(), "0.1".into()).checked_apply(&mut canvas)?;
    // create higher band 2
    Move::Merge("0.3".into(), "0.2".into()).checked_apply(&mut canvas)?;
    // create whole canvas 3
    Move::Merge("1".into(), "2".into()).checked_apply(&mut canvas)?;
    Move::PointCut("3".into(), 16, 16).checked_apply(&mut canvas)?;
    let ref_canvas = make_cross_canvas();
    assert_eq!(&canvas, &ref_canvas);
    Ok(())
}

#[test]
fn line_cut() -> Result<(), MoveError> {
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let mut canvas = Canvas::new(32, 32);
        Move::LineCut("0".into(), orientation, 16).checked_apply(&mut canvas)?;
    }
    Ok(())
}

#[test]
fn test_color() -> Result<(), MoveError> {
    let mut canvas = make_complicated_canvas();
    Move::Color("0.0".into(), Color::new(1, 2, 3, 4)).checked_apply(&mut canvas)?;
    Move::Color("1".into(), Color::new(2, 2, 3, 4)).checked_apply(&mut canvas)?;
    Ok(())
}
