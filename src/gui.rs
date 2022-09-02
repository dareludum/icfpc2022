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

pub fn gui_main(problem_path: &std::path::Path) {
    let painting = Painting::load(problem_path);
    let canvas = Canvas::new(painting.width(), painting.height());

    let (mut rl, thread) = raylib::init()
        .size(1000, 600)
        .title("ICFPC2022 - dare ludum")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        const MARGIN: i32 = 20;
        const IMAGE_SIZE: i32 = 400;

        // Draw the in-progress solution
        for b in canvas.blocks_iter() {
            for x in b.r.x..b.r.x + b.r.w {
                for y in b.r.y..b.r.y + b.r.h {
                    d.draw_pixel(MARGIN + x as i32, MARGIN + y as i32, b.c);
                }
            }
        }

        // Draw the target
        for x in 0..painting.width() {
            for y in 0..painting.height() {
                let c = painting.get_color(x, y);
                d.draw_pixel(
                    MARGIN + IMAGE_SIZE + MARGIN + x as i32,
                    MARGIN + y as i32,
                    c,
                )
            }
        }
    }
}
