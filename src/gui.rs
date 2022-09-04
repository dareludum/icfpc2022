#![allow(clippy::collapsible_if)]
use std::fmt::Write;

use colorgrad::Gradient;
use raylib::prelude::*;

use crate::{
    block::{BlockData, Point, Rect},
    canvas::Canvas,
    moves::{AppliedMove, Cost, Move, Orientation, UndoMove},
    painting::Painting,
    program::{self, to_isl},
};

impl From<crate::color::Color> for raylib::ffi::Color {
    fn from(c: crate::color::Color) -> Self {
        raylib::prelude::Color::new(c.r, c.g, c.b, c.a).into()
    }
}

struct GradientColor(colorgrad::Color);
impl From<GradientColor> for raylib::ffi::Color {
    fn from(GradientColor(c): GradientColor) -> Self {
        let c = c.to_rgba8();
        raylib::prelude::Color::new(c[0], c[1], c[2], c[3]).into()
    }
}

#[derive(PartialEq, Eq)]
enum Tool {
    CutVert,
    CutHorz,
    CutCross,
    Color,
    Swap,
    Merge,
}

impl Tool {
    pub fn name(&self) -> &'static str {
        match self {
            Tool::CutVert => "cut vert",
            Tool::CutHorz => "cut horz",
            Tool::CutCross => "cut cross",
            Tool::Color => "color",
            Tool::Swap => "swap",
            Tool::Merge => "merge",
        }
    }
}

type Offset = (i32, i32);

const MARGIN: i32 = 20;
const IMAGE_SIZE: i32 = 400;
const COLOR_PREVIEW_SIZE: i32 = 50;
const SLN: Offset = (MARGIN, MARGIN);
const TGT: Offset = (MARGIN + IMAGE_SIZE + MARGIN, MARGIN);

const COLOR_BLOCK_BORDER: Color = Color {
    a: 32,
    ..Color::GRAY
};
const COLOR_WORST_BLOCK: Color = Color::RED;

const SOLUTION_RECT: Rect = Rect::new(
    Point::new(MARGIN as u32, MARGIN as u32),
    Point::new((MARGIN + IMAGE_SIZE) as u32, (MARGIN + IMAGE_SIZE) as u32),
);
const TARGET_RECT: Rect = Rect::new(
    Point::new((MARGIN * 2 + IMAGE_SIZE) as u32, MARGIN as u32),
    Point::new(
        (MARGIN * 2 + IMAGE_SIZE * 2) as u32,
        (MARGIN + IMAGE_SIZE) as u32,
    ),
);

pub fn gui_main(problem_path: &std::path::Path) {
    let painting = Painting::load(problem_path);
    let initial_config_path = problem_path.with_extension("json");
    let mut canvas = Canvas::try_create(initial_config_path, &painting)
        .expect("gui_main: Error while creating Canvas");
    let initial_painting_score = painting.calculate_score_canvas(&canvas);
    let mut current_painting_score = initial_painting_score;
    let mut current_tool_score = Cost(0);
    let mut current_worst_block_id = painting.find_worst_block_id(&canvas).clone();
    let mut show_alt_data = false;

    let (mut rl, thread) = raylib::init()
        .size(1080, 600)
        .title("ICFPC2022 - dare ludum")
        .build();

    let width = painting.width() as i32;
    let height = painting.height() as i32;
    let mut target_image = Image::gen_image_color(width, height, Color::BLACK);
    for x in 0..painting.width() {
        for y in 0..painting.height() {
            let c = painting.get_color(x, y);
            target_image.draw_pixel(x as i32, y as i32, c);
        }
    }
    let target_texture = rl.load_texture_from_image(&thread, &target_image).unwrap();
    let abs_diff_gradient = colorgrad::CustomGradient::new()
        .colors(&[
            colorgrad::Color::from_rgba8(255, 255, 255, 255),
            colorgrad::Color::from_rgba8(255, 0, 0, 255),
        ])
        .domain(&[0.0, 500.0])
        .build()
        .unwrap();
    let mut current_abs_diff_texture = rl
        .load_texture_from_image(
            &thread,
            &generate_abs_diff_image(&canvas, &painting, &abs_diff_gradient),
        )
        .unwrap();

    let mut tool = Tool::CutVert;
    let mut color = crate::color::Color::new(255, 255, 255, 255);
    let mut marked_block = None;
    let mut moves: Vec<AppliedMove> = vec![];

    while !rl.window_should_close() {
        // ===== HIT TEST =====
        let mut mx = rl.get_mouse_x();
        let my = rl.get_mouse_y();

        // Hack to force the logical mouse coords to always "be" inside the solution
        if TARGET_RECT.contains(mx as u32, my as u32) {
            mx -= IMAGE_SIZE + MARGIN;
        }

        let mut b_id = if SOLUTION_RECT.contains(mx as u32, my as u32) {
            match tool {
                Tool::CutHorz | Tool::CutVert | Tool::CutCross => {
                    rl.hide_cursor();
                }
                _ => {}
            }
            Some(canvas.hit_test((mx - MARGIN) as u32, (my - MARGIN) as u32))
        } else {
            rl.show_cursor();
            None
        };

        // ===== INTERACTION =====
        match rl.get_key_pressed() {
            Some(k) => match k {
                KeyboardKey::KEY_ONE => {
                    tool = if tool == Tool::CutVert {
                        Tool::CutHorz
                    } else {
                        Tool::CutVert
                    };
                    rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_ARROW);
                    rl.show_cursor();
                }
                KeyboardKey::KEY_TWO => {
                    tool = Tool::CutCross;
                    rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_ARROW);
                    rl.show_cursor();
                }
                KeyboardKey::KEY_THREE => {
                    tool = Tool::Color;
                    rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_CROSSHAIR);
                    rl.show_cursor();
                }
                KeyboardKey::KEY_FOUR => {
                    tool = Tool::Swap;
                    rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_POINTING_HAND);
                    rl.show_cursor();
                }
                KeyboardKey::KEY_FIVE => {
                    tool = Tool::Merge;
                    rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_POINTING_HAND);
                    rl.show_cursor();
                }
                KeyboardKey::KEY_U => {
                    if let Some(applied_move) = moves.pop() {
                        // WTF?
                        let undo: UndoMove = applied_move.undo;
                        undo.apply(&mut canvas);
                        b_id = None;
                        marked_block = None;
                        current_painting_score = painting.calculate_score_canvas(&canvas);
                        current_tool_score -= applied_move.cost;
                        current_worst_block_id = painting.find_worst_block_id(&canvas).clone();
                        current_abs_diff_texture = rl
                            .load_texture_from_image(
                                &thread,
                                &generate_abs_diff_image(&canvas, &painting, &abs_diff_gradient),
                            )
                            .unwrap();
                    }
                }
                KeyboardKey::KEY_S => {
                    let mut result = vec![];
                    for am in &moves {
                        result.push(am.mov.clone());
                    }
                    program::write_to_file(&std::path::PathBuf::from("./manual.txt"), &result)
                        .expect("Failed to write the manual solution file");
                }
                KeyboardKey::KEY_TAB => {
                    show_alt_data = !show_alt_data;
                }
                _ => {}
            },
            None => {}
        }

        // TODO refactor to a function
        let mov = if let Some(b_id) = b_id.clone() {
            match tool {
                Tool::CutVert => Some(Move::LineCut(
                    b_id.clone(),
                    Orientation::Vertical,
                    (mx - SLN.0) as u32,
                )),
                Tool::CutHorz => Some(Move::LineCut(
                    b_id.clone(),
                    Orientation::Horizontal,
                    (my - SLN.1) as u32,
                )),
                Tool::CutCross => Some(Move::PointCut(
                    b_id.clone(),
                    (mx - SLN.0) as u32,
                    (my - SLN.1) as u32,
                )),
                Tool::Color => Some(Move::Color(b_id.clone(), color)),
                Tool::Swap => marked_block.clone().map(|id| Move::Swap(b_id, id)),
                Tool::Merge => marked_block.clone().map(|id| Move::Merge(b_id, id)),
            }
        } else {
            None
        };

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
            if let Some(mov) = mov.clone() {
                if let Ok(applied_move) = mov.apply(&mut canvas) {
                    b_id = None;
                    marked_block = None;
                    current_painting_score = painting.calculate_score_canvas(&canvas);
                    current_tool_score += applied_move.cost;
                    current_worst_block_id = painting.find_worst_block_id(&canvas).clone();
                    current_abs_diff_texture = rl
                        .load_texture_from_image(
                            &thread,
                            &generate_abs_diff_image(&canvas, &painting, &abs_diff_gradient),
                        )
                        .unwrap();
                    moves.push(applied_move);
                } else {
                    // TODO: show a message
                }
            }
        }

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_RIGHT_BUTTON) {
            if let Some(b_id) = b_id.clone() {
                match tool {
                    Tool::CutVert => {}
                    Tool::CutHorz => {}
                    Tool::CutCross => {}
                    Tool::Color => {
                        color = painting.get_color(
                            mx as u32 - SOLUTION_RECT.x(),
                            my as u32 - SOLUTION_RECT.y(),
                        );
                    }
                    Tool::Swap | Tool::Merge => {
                        marked_block = Some(b_id);
                    }
                }
            }
        }

        // ===== DRAWING =====
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        // Draw the borders
        double_draw_rectangle_lines(
            &mut d,
            MARGIN - 1,
            MARGIN - 1,
            IMAGE_SIZE + 2,
            IMAGE_SIZE + 2,
            Color::BLACK,
        );

        // Draw the target
        d.draw_texture(
            if show_alt_data {
                &current_abs_diff_texture
            } else {
                &target_texture
            },
            MARGIN + IMAGE_SIZE + MARGIN,
            MARGIN,
            Color::WHITE,
        );

        // Draw the in-progress solution
        for b in canvas.blocks_iter() {
            let id = &b.id;
            match &b.data {
                BlockData::Simple(c) => {
                    d.draw_rectangle(
                        MARGIN + b.r.bottom_left.x as i32,
                        MARGIN + b.r.bottom_left.y as i32,
                        b.r.width() as i32,
                        b.r.height() as i32,
                        *c,
                    );
                    double_draw_rectangle_lines(
                        &mut d,
                        MARGIN + b.r.bottom_left.x as i32,
                        MARGIN + b.r.bottom_left.y as i32,
                        b.r.width() as i32,
                        b.r.height() as i32,
                        if id == &current_worst_block_id {
                            COLOR_WORST_BLOCK
                        } else {
                            COLOR_BLOCK_BORDER
                        },
                    );
                }
                BlockData::Complex(bs) => {
                    for b in bs {
                        d.draw_rectangle(
                            MARGIN + b.r.bottom_left.x as i32,
                            MARGIN + b.r.bottom_left.y as i32,
                            b.r.width() as i32,
                            b.r.height() as i32,
                            b.c,
                        );
                    }
                    double_draw_rectangle_lines(
                        &mut d,
                        MARGIN + b.r.bottom_left.x as i32,
                        MARGIN + b.r.bottom_left.y as i32,
                        b.r.width() as i32,
                        b.r.height() as i32,
                        if id == &current_worst_block_id {
                            COLOR_WORST_BLOCK
                        } else {
                            COLOR_BLOCK_BORDER
                        },
                    );
                }
            }
        }

        // Draw the overlays
        if let Some(b_id) = b_id.clone() {
            let x = mx - MARGIN;
            let y = my - MARGIN;
            let b = canvas.get_block(&b_id).unwrap();
            let r = &b.r;
            double_draw_rectangle_lines(
                &mut d,
                MARGIN + r.bottom_left.x as i32,
                MARGIN + r.bottom_left.y as i32,
                r.width() as i32,
                r.height() as i32,
                Color::GREEN,
            );
            if let Some(mb) = marked_block.clone() {
                let mr = &canvas.get_block(&mb).unwrap().r;
                double_draw_rectangle_lines(
                    &mut d,
                    MARGIN + mr.bottom_left.x as i32,
                    MARGIN + mr.bottom_left.y as i32,
                    mr.width() as i32,
                    mr.height() as i32,
                    Color::PURPLE,
                );
            }
            match tool {
                Tool::CutVert => {
                    draw_notch_vert(&mut d, &SLN, x, r.y() as i32, r.height() as i32, Color::RED);
                    draw_notch_vert(&mut d, &TGT, x, r.y() as i32, r.height() as i32, Color::RED);
                }
                Tool::CutHorz => {
                    draw_notch_horz(&mut d, &SLN, r.x() as i32, y, r.width() as i32, Color::RED);
                    draw_notch_horz(&mut d, &TGT, r.x() as i32, y, r.width() as i32, Color::RED);
                }
                Tool::CutCross => {
                    draw_notch_vert(&mut d, &SLN, x, r.y() as i32, r.height() as i32, Color::RED);
                    draw_notch_vert(&mut d, &TGT, x, r.y() as i32, r.height() as i32, Color::RED);
                    draw_notch_horz(&mut d, &SLN, r.x() as i32, y, r.width() as i32, Color::RED);
                    draw_notch_horz(&mut d, &TGT, r.x() as i32, y, r.width() as i32, Color::RED);
                }
                Tool::Color => {}
                Tool::Swap => {}
                Tool::Merge => {}
            }
        }

        // Draw info
        d.draw_rectangle(
            MARGIN,
            MARGIN + IMAGE_SIZE + MARGIN,
            COLOR_PREVIEW_SIZE,
            COLOR_PREVIEW_SIZE,
            color,
        );

        d.draw_text(
            &format!(
                "Score: {} (initial), {} (current, {} + {})",
                initial_painting_score.0,
                (current_painting_score.0 + current_tool_score.0),
                current_painting_score.0,
                current_tool_score.0
            ),
            MARGIN + COLOR_PREVIEW_SIZE + MARGIN,
            MARGIN + IMAGE_SIZE + MARGIN,
            20,
            Color::BLACK,
        );

        d.draw_text(
            &format!("Tool: {}", tool.name()),
            MARGIN + COLOR_PREVIEW_SIZE + MARGIN,
            MARGIN + IMAGE_SIZE + 2 * MARGIN,
            20,
            Color::BLACK,
        );

        if let Some(mv) = mov {
            d.draw_text(
                &format!("Move: {}", to_isl(&mv)),
                MARGIN + COLOR_PREVIEW_SIZE + MARGIN,
                MARGIN + IMAGE_SIZE + 3 * MARGIN,
                20,
                Color::BLACK,
            );
        }

        const MAX_MOVES: usize = 12;
        let mut s = "Moves:\n".to_string();
        for am in moves.iter().rev().take(MAX_MOVES) {
            writeln!(s, "{}", to_isl(&am.mov)).unwrap();
        }
        if moves.len() > MAX_MOVES {
            s.push_str("...")
        }

        d.draw_text(
            &s,
            MARGIN + IMAGE_SIZE + MARGIN + IMAGE_SIZE + MARGIN,
            MARGIN,
            20,
            Color::BLACK,
        );
    }
}

fn generate_abs_diff_image(canvas: &Canvas, painting: &Painting, gradient: &Gradient) -> Image {
    let width = painting.width() as i32;
    let height = painting.height() as i32;
    let mut target_image = Image::gen_image_color(width, height, Color::BLACK);
    let abs_diff_map = painting.calculate_abs_diff_map(canvas);
    for x in 0..painting.width() {
        for y in 0..painting.height() {
            let v = abs_diff_map[(x + (y * painting.width)) as usize];
            let c = GradientColor(gradient.at(v));
            target_image.draw_pixel(x as i32, y as i32, c);
        }
    }
    target_image
}

fn double_draw_rectangle_lines(
    d: &mut RaylibDrawHandle,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: Color,
) {
    d.draw_rectangle_lines(x, y, width, height, color);
    d.draw_rectangle_lines(x + IMAGE_SIZE + MARGIN, y, width, height, color);
}

fn draw_notch_horz(
    d: &mut RaylibDrawHandle,
    offset: &Offset,
    x: i32,
    y: i32,
    width: i32,
    color: Color,
) {
    draw_line_horz(d, offset, x, y - 1, width, color);
    draw_line_horz(d, offset, x, y + 1, width, color);
}

fn draw_notch_vert(
    d: &mut RaylibDrawHandle,
    offset: &Offset,
    x: i32,
    y: i32,
    height: i32,
    color: Color,
) {
    draw_line_vert(d, offset, x - 1, y, height, color);
    draw_line_vert(d, offset, x + 1, y, height, color);
}

fn draw_line_horz(
    d: &mut RaylibDrawHandle,
    offset: &Offset,
    x: i32,
    y: i32,
    width: i32,
    color: Color,
) {
    d.draw_line(
        offset.0 + x,
        offset.1 + y,
        offset.0 + x + width,
        offset.1 + y,
        color,
    );
}

fn draw_line_vert(
    d: &mut RaylibDrawHandle,
    offset: &Offset,
    x: i32,
    y: i32,
    height: i32,
    color: Color,
) {
    d.draw_line(
        offset.0 + x,
        offset.1 + y,
        offset.0 + x,
        offset.1 + y + height,
        color,
    );
}
