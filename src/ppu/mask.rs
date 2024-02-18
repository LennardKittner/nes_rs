use bitflags::bitflags;

bitflags! {
    // 7  bit  0
    // ---- ----
    // BGRs bMmG
    // |||| ||||
    // |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
    // |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
    // |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    // |||| +---- 1: Show background
    // |||+------ 1: Show sprites
    // ||+------- Emphasize red (green on PAL/Dendy)
    // |+-------- Emphasize green (red on PAL/Dendy)
    // +--------- Emphasize blue
    pub struct MaskRegister: u8 {
        const GREYSCALE               = 0b0000_0001;
        const BACKGROUND_LEFT = 0b0000_0010;
        const SPRITES_LEFT = 0b0000_0100;
        const BACKGROUND = 0b0000_1000;
        const SPRITES = 0b0001_0000;
        const RED = 0b0010_0000;
        const GREEN = 0b0100_0000;
        const BLUE = 0b1000_0000;
    }
}

pub enum Color {
    Red,
    Green,
    Blue
}

impl MaskRegister {
    pub fn new() -> Self {
        Self::empty()
    }

    pub fn update(&mut self, data: u8) {
        self.0 = Self::from_bits(data).unwrap().0;
    }

    pub fn get_emphesis(&self) -> Vec<Color> {
        let mut result = Vec::new();
        if self.contains(Self::RED) {
            result.push(Color::Red);
        }
        if self.contains(Self::GREEN) {
            result.push(Color::Green);
        }
        if self.contains(Self::BLUE) {
            result.push(Color::Blue);
        }
        result
    }
}
