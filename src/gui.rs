use raylib::prelude::*;

use crate::{
    block::{Point, Rect},
    canvas::Canvas,
    painting::{self, Painting},
};

impl From<crate::block::Color> for raylib::ffi::Color {
    fn from(c: crate::block::Color) -> Self {
        raylib::prelude::Color::new(c.r, c.g, c.b, c.a).into()
    }
}

#[derive(PartialEq, Eq)]
enum Tool {
    CutHorz,
    CutVert,
    CutCross,
    Color,
    Swap,
    Merge,
}

impl Tool {
    pub fn name(&self) -> &'static str {
        match self {
            Tool::CutHorz => "cut horz",
            Tool::CutVert => "cut vert",
            Tool::CutCross => "cut cross",
            Tool::Color => "color",
            Tool::Swap => "swap",
            Tool::Merge => "merge",
        }
    }
}

type Offset = (i32, i32);

pub fn gui_main(problem_path: &std::path::Path) {
    let painting = Painting::load(problem_path);
    let canvas = Canvas::new(painting.width(), painting.height());

    let (mut rl, thread) = raylib::init()
        .size(1000, 600)
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

    let mut tool = Tool::CutHorz;
    let mut color = crate::block::Color::new(255, 255, 255, 255);

    const MARGIN: i32 = 20;
    const IMAGE_SIZE: i32 = 400;
    const COLOR_PREVIEW_SIZE: i32 = 50;
    // const OFFSET_SOLUTION: Offset = (0, MARGIN);
    // const OFFSET_TARGET: Offset = (MARGIN + IMAGE_SIZE, MARGIN);
    const ZERO: Offset = (MARGIN, MARGIN);
    const TGT: Offset = (MARGIN + IMAGE_SIZE + MARGIN, MARGIN);
    const SOLUTION_FROM_TARGET: Offset = (-(IMAGE_SIZE + MARGIN), MARGIN);

    const SOLUTION_RECT: Rect = Rect::new(
        Point::new(MARGIN as u32, MARGIN as u32),
        Point::new((MARGIN + IMAGE_SIZE) as u32, (MARGIN + IMAGE_SIZE) as u32),
    );
    const TARGET_RECT: Rect = Rect::new(
        Point::new((MARGIN * 2 + IMAGE_SIZE) as u32, MARGIN as u32),
        Point::new((MARGIN * 2 + IMAGE_SIZE * 2) as u32, (MARGIN + IMAGE_SIZE) as u32),
    );

    while !rl.window_should_close() {
        // ===== HIT TEST =====
        let mx = rl.get_mouse_x();
        let my = rl.get_mouse_y();
        let block =
            if mx >= MARGIN && mx < MARGIN + IMAGE_SIZE && my >= MARGIN && my < MARGIN + IMAGE_SIZE
            {
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
                    tool = if tool == Tool::CutHorz {
                        Tool::CutVert
                    } else {
                        Tool::CutHorz
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
                _ => {}
            },
            None => {}
        }

        if rl.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
            if SOLUTION_RECT.contains(mx as u32, my as u32) {
                match tool {
                    Tool::CutHorz => {}
                    Tool::CutVert => {}
                    Tool::CutCross => {}
                    Tool::Color => {}
                    Tool::Swap => {}
                    Tool::Merge => {}
                }
            } else if TARGET_RECT.contains(mx as u32, my as u32) {
                match tool {
                    Tool::CutHorz => {}
                    Tool::CutVert => {}
                    Tool::CutCross => {}
                    Tool::Color => {
                        color = painting
                            .get_color(mx as u32 - TARGET_RECT.x(), my as u32 - TARGET_RECT.y());
                    }
                    Tool::Swap => {}
                    Tool::Merge => {}
                }
            }
        }

        // ===== DRAWING =====
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        // Draw the borders
        d.draw_rectangle_lines(
            MARGIN - 1,
            MARGIN - 1,
            IMAGE_SIZE + 2,
            IMAGE_SIZE + 2,
            Color::BLACK,
        );
        d.draw_rectangle_lines(
            MARGIN + IMAGE_SIZE + MARGIN - 1,
            MARGIN - 1,
            IMAGE_SIZE + 2,
            IMAGE_SIZE + 2,
            Color::BLACK,
        );

        // Draw the in-progress solution
        for b in canvas.blocks_iter() {
            d.draw_rectangle(
                MARGIN + b.r.bottom_left.x as i32,
                MARGIN + b.r.bottom_left.y as i32,
                b.r.width() as i32,
                b.r.height() as i32,
                b.c,
            );
        }

        // Draw the target
        d.draw_texture(
            &target_texture,
            MARGIN + IMAGE_SIZE + MARGIN,
            MARGIN,
            Color::WHITE,
        );

        // Draw the overlays
        if let Some(b) = block {
            let x = mx - MARGIN;
            let y = my - MARGIN;
            let r = b.rect();
            d.draw_rectangle_lines(
                MARGIN + r.bottom_left.x as i32,
                MARGIN + r.bottom_left.y as i32,
                r.width() as i32,
                r.height() as i32,
                Color::GREEN,
            );
            match tool {
                Tool::CutHorz => {
                    draw_notch_vert(&mut d, &ZERO, x, r.y() as i32, r.height() as i32, Color::RED);
                    draw_notch_vert(&mut d, &TGT, x, r.y() as i32, r.height() as i32, Color::RED);
                }
                Tool::CutVert => {
                    draw_notch_horz(&mut d, &ZERO, r.x() as i32, y, r.width() as i32, Color::RED);
                    draw_notch_horz(&mut d, &TGT, r.x() as i32, y, r.width() as i32, Color::RED);
                }
                Tool::CutCross => {
                    draw_notch_vert(&mut d, &ZERO, x, r.y() as i32, r.height() as i32, Color::RED);
                    draw_notch_vert(&mut d, &TGT, x, r.y() as i32, r.height() as i32, Color::RED);
                    draw_notch_horz(&mut d, &ZERO, r.x() as i32, y, r.width() as i32, Color::RED);
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
            &format!("Tool: {}", tool.name()),
            MARGIN + COLOR_PREVIEW_SIZE + MARGIN,
            MARGIN + IMAGE_SIZE + MARGIN,
            20,
            Color::BLACK,
        );
    }
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
