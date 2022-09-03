use super::*;
use crate::block::*;
use crate::moves::HistoryTester;

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
    return Canvas::from_blocks(32, 32, 2, blocks.into_iter());
}

#[test]
fn test_complicated() -> Result<(), MoveError> {
    let mut tester = HistoryTester::new(Canvas::new(32, 32));
    tester.apply(Move::LineCut("0".to_owned(), Orientation::Vertical, 16))?;
    tester.apply(Move::PointCut("0.1".to_owned(), 24, 16))?;
    tester.apply(Move::Merge("0.1.2".to_owned(), "0.1.3".to_owned()))?;
    let ref_canvas = make_test_canvas();
    assert_eq!(tester.get_canvas(), &ref_canvas);
    tester.validate_history();
    Ok(())
}

#[test]
fn line_cut() -> Result<(), MoveError> {
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let mut canvas = Canvas::new(32, 32);
        let bare_canvas = canvas.clone();
        let (_cost, undo) = Move::LineCut("0".to_owned(), orientation, 16).apply(&mut canvas)?;
        undo.apply(&mut canvas);
        assert_eq!(canvas, bare_canvas);
    }
    Ok(())
}
