struct Rectangle {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

enum Block {
    SimpleBlock(Rectangle, Color),
    ComplexBlock(Rectangle, Vec<Block>),
}
