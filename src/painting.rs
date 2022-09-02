use crate::block::Color;

pub struct Painting {

}

impl Painting {
    pub fn load(path: &std::path::Path) -> Self {
        todo!()
    }

    pub fn width(&self) -> u32 {
        todo!()
    }

    pub fn height(&self) -> u32 {
        todo!()
    }

    pub fn get_color(&self, x: u32, y: u32) -> Color {
        todo!()
    }

    pub fn calculate_score(&self, target: &Painting) -> u32 {
        todo!()
    }
}
