use raylib::prelude::*;

use crate::{
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

    while !rl.window_should_close() {
        const MARGIN: i32 = 20;
        const IMAGE_SIZE: i32 = 400;

        // ===== HIT TEST =====
        let mx = rl.get_mouse_x();
        let my = rl.get_mouse_y();
        let block =
            if mx >= MARGIN && mx < MARGIN + IMAGE_SIZE && my >= MARGIN && my < MARGIN + IMAGE_SIZE
            {
                Some(canvas.hit_test((mx - MARGIN) as u32, (my - MARGIN) as u32))
            } else {
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
                }
                KeyboardKey::KEY_TWO => {
                    tool = Tool::CutCross;
                }
                KeyboardKey::KEY_THREE => {
                    tool = Tool::Color;
                }
                KeyboardKey::KEY_FOUR => {
                    tool = Tool::Swap;
                }
                KeyboardKey::KEY_FIVE => {
                    tool = Tool::Merge;
                }
                _ => {}
            },
            None => {}
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
                MARGIN + b.r.x as i32,
                MARGIN + b.r.y as i32,
                b.r.width() as i32,
                b.r.height() as i32,
                b.c,
            );
        }

        // Draw solution overlays
        if let Some(b) = block {
            let r = b.rect();
            d.draw_rectangle_lines(
                MARGIN + r.x as i32,
                MARGIN + r.y as i32,
                r.width() as i32,
                r.height() as i32,
                Color::GREEN,
            );
            match tool {
                Tool::CutHorz => {
                    d.draw_line(
                        mx,
                        MARGIN + r.y as i32,
                        mx,
                        MARGIN + r.y as i32 + r.height() as i32,
                        Color::RED,
                    );
                }
                Tool::CutVert => {}
                Tool::CutCross => {}
                Tool::Color => {}
                Tool::Swap => {}
                Tool::Merge => {}
            }
        }

        // Draw the target
        d.draw_texture(
            &target_texture,
            MARGIN + IMAGE_SIZE + MARGIN,
            MARGIN,
            Color::WHITE,
        );

        // Draw info
        d.draw_text(
            &format!("Tool: {}", tool.name()),
            MARGIN,
            MARGIN + IMAGE_SIZE + MARGIN,
            20,
            Color::BLACK,
        );
    }
}
