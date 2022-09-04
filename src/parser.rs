use crate::block::BlockId;
use crate::color::Color;
use crate::moves::{Move, Orientation};
use nom::character::complete::{newline, one_of};
use nom::multi::many1;
use nom::sequence::{preceded, separated_pair, terminated};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::char,
    combinator::recognize,
    sequence::{delimited, tuple},
    IResult,
};

fn group_str(input: &str) -> IResult<&str, &str> {
    delimited(char('['), take_until("]"), char(']'))(input)
}

fn group_block_id(input: &str) -> IResult<&str, BlockId> {
    let (rem, id) = group_str(input)?;
    Ok((rem, BlockId::new(id.into())))
}

fn integer(input: &str) -> IResult<&str, u32> {
    let (rem, number_str) = recognize(many1(one_of("0123456789")))(input)?;
    let number: u32 = lexical_core::parse(number_str.as_bytes()).unwrap();
    Ok((rem, number))
}

fn group_integer(input: &str) -> IResult<&str, u32> {
    delimited(char('['), integer, char(']'))(input)
}

fn group_color(input: &str) -> IResult<&str, Color> {
    let comps = tuple((
        integer,
        preceded(char(','), integer),
        preceded(char(','), integer),
        preceded(char(','), integer),
    ));
    let (rem, (r, g, b, a)) = delimited(char('['), comps, char(']'))(input)?;
    Ok((rem, Color::new(r as u8, g as u8, b as u8, a as u8)))
}

fn group_point(input: &str) -> IResult<&str, (u32, u32)> {
    delimited(
        char('['),
        separated_pair(integer, char(','), integer),
        char(']'),
    )(input)
}

fn group_orientation(input: &str) -> IResult<&str, Orientation> {
    let (rem, orient_str) = delimited(char('['), one_of("xy"), char(']'))(input)?;
    Ok((
        rem,
        match orient_str {
            'x' => Orientation::Vertical,
            'y' => Orientation::Horizontal,
            _ => panic!(),
        },
    ))
}

fn move_line_cut(input: &str) -> IResult<&str, Move> {
    let res = tuple((tag("cut"), group_block_id, group_orientation, group_integer))(input)?;
    let (rem, (_, id, orientation, offset)) = res;
    Ok((rem, Move::LineCut(id, orientation, offset)))
}

fn move_point_cut(input: &str) -> IResult<&str, Move> {
    let res = tuple((tag("cut"), group_block_id, group_point))(input)?;
    let (rem, (_, id, (x, y))) = res;
    Ok((rem, Move::PointCut(id, x, y)))
}

fn move_color(input: &str) -> IResult<&str, Move> {
    let res = tuple((tag("color"), group_block_id, group_color))(input)?;
    let (rem, (_, id, color)) = res;
    Ok((rem, Move::Color(id, color)))
}

fn move_merge(input: &str) -> IResult<&str, Move> {
    let res = tuple((tag("merge"), group_block_id, group_block_id))(input)?;
    let (rem, (_, id_a, id_b)) = res;
    Ok((rem, Move::Merge(id_a, id_b)))
}

fn move_swap(input: &str) -> IResult<&str, Move> {
    let res = tuple((tag("swap"), group_block_id, group_block_id))(input)?;
    let (rem, (_, id_a, id_b)) = res;
    Ok((rem, Move::Swap(id_a, id_b)))
}

fn parse_move(input: &str) -> IResult<&str, Move> {
    alt((
        move_line_cut,
        move_point_cut,
        move_color,
        move_merge,
        move_swap,
    ))(input)
}

pub fn parse_move_line(input: &str) -> IResult<&str, Move> {
    terminated(parse_move, newline)(input)
}

#[test]
fn test_parse_isl() {
    fn assert_isl(line: &str, mov: Move) {
        match parse_move(line) {
            Ok((remainder, result)) => {
                assert_eq!(remainder, "", "expected an empty remainder");
                assert_eq!(&result, &mov);
            }
            Err(e) => panic!("failed to parse {line} to {mov:?}: {e}"),
        }
    }

    assert_isl("merge[1][2]", Move::Merge("1".into(), "2".into()));
    assert_isl(
        "swap[3.0.0][0.0.1]",
        Move::Swap("3.0.0".into(), "0.0.1".into()),
    );
    assert_isl(
        "color[1.1][255,255,255,0]",
        Move::Color("1.1".into(), Color::new(255, 255, 255, 0)),
    );
    assert_isl(
        "cut[0][x][12]",
        Move::LineCut("0".into(), Orientation::Vertical, 12),
    );
    assert_isl(
        "cut[2][y][1]",
        Move::LineCut("2".into(), Orientation::Horizontal, 1),
    );
    assert_isl("cut[1][44,44]", Move::PointCut("1".into(), 44, 44));
}
