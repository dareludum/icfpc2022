use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{},{})", self.r, self.g, self.b, self.a)
    }
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub fn find_most_common(counts: &HashMap<Color, u32>) -> Self {
        *counts.iter().max_by_key(|(_, v)| *v).unwrap().0
    }

    pub fn find_average(counts: &HashMap<Color, u32>) -> Self {
        assert!(!counts.is_empty());
        let total_pixels = counts.iter().map(|(_, v)| *v as u64).sum::<u64>();
        let mut r = 0u64;
        let mut g = 0u64;
        let mut b = 0u64;
        let mut a = 0u64;
        for (c, cnt) in counts {
            r += c.r as u64 * *cnt as u64;
            g += c.g as u64 * *cnt as u64;
            b += c.b as u64 * *cnt as u64;
            a += c.a as u64 * *cnt as u64;
        }
        Color {
            r: (r / total_pixels) as u8,
            g: (g / total_pixels) as u8,
            b: (b / total_pixels) as u8,
            a: (a / total_pixels) as u8,
        }
    }
}
