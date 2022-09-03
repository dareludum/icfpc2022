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
fn make_test_canvas() -> Canvas {
    let bg = Color::new(255, 255, 255, 255);
    let mut blocks: Vec<Block> = vec![];
    blocks.push(SimpleBlock::new("0.0".into(), Rect::from_coords([0, 0, 16, 32]), bg).into());
    blocks.push(SimpleBlock::new("0.1.0".into(), Rect::from_coords([16, 0, 24, 16]), bg).into());
    blocks.push(SimpleBlock::new("0.1.1".into(), Rect::from_coords([24, 0, 32, 16]), bg).into());
    blocks.push(
        ComplexBlock::new(
            "1".into(),
            Rect::from_coords([16, 16, 32, 32]),
            vec![
                SimpleBlock::new("0.1.2".into(), Rect::from_coords([24, 16, 32, 32]), bg),
                SimpleBlock::new("0.1.3".into(), Rect::from_coords([16, 16, 24, 32]), bg),
            ],
        )
        .into(),
    );
    // this is a 3rd generation canvas, as 3 moves were applied
    return Canvas::from_blocks(32, 32, 2, 3, blocks.into_iter());
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
    let mut canvas = make_test_canvas();
    Move::Color("0.0".into(), Color::new(1, 2, 3, 4)).checked_apply(&mut canvas)?;
    Move::Color("1".into(), Color::new(2, 2, 3, 4)).checked_apply(&mut canvas)?;
    Ok(())
}

#[test]
fn test_complicated() -> Result<(), MoveError> {
    let mut canvas = Canvas::new(32, 32);
    Move::LineCut("0".into(), Orientation::Vertical, 16).checked_apply(&mut canvas)?;
    Move::PointCut("0.1".into(), 24, 16).checked_apply(&mut canvas)?;
    Move::Merge("0.1.2".into(), "0.1.3".into()).checked_apply(&mut canvas)?;
    let ref_canvas = make_test_canvas();
    assert_eq!(&canvas, &ref_canvas);
    Ok(())
}
