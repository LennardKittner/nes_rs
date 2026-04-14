use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct Sprite {
    y_pos: u8,
    pattern_index: u8,
    attributes: u8,
    x_pos: u8,
    sprite_zero: bool,
    /// whether this sprite is the upper half in 8x16 mode
    upper_half: bool,
}

impl Sprite {
    /// create a new sprite with the raw data whether the sprite is sprite zero and whether it is
    /// the upper half in 8x16 mode
    pub fn new(raw: &[u8], sprite_zero: bool, upper_half: bool) -> Option<Self> {
        if raw.len() < 4 {
            None
        } else {
            Some(Sprite {
                y_pos: raw[0],
                pattern_index: raw[1],
                attributes: raw[2],
                x_pos: raw[3],
                sprite_zero,
                upper_half,
            })
        }
    }

    pub fn get_x(&self) -> usize {
        self.x_pos as usize
    }

    pub fn get_y(&self) -> usize {
        self.y_pos as usize
    }

    pub fn get_pattern_index(&self) -> usize {
        self.pattern_index as usize
    }

    pub fn is_horizontal_flip(&self) -> bool {
        (self.attributes >> 6) & 1 == 1
    }

    pub fn is_vertical_flip(&self) -> bool {
        self.attributes >> 7 == 1
    }

    pub fn draw_over_background(&self) -> bool {
        (self.attributes >> 5) & 1 != 1
    }

    pub fn get_palette_index(&self) -> usize {
        (self.attributes & 0b11) as usize
    }

    pub fn is_sprite_zero(&self) -> bool {
        self.sprite_zero
    }

    pub fn is_upper_half(&self) -> bool {
        self.upper_half
    }

    pub fn get_raw_data(&self) -> [u8; 4] {
        [self.y_pos, self.pattern_index, self.attributes, self.x_pos]
    }

    pub fn set_y(&mut self, y_pos: u8) {
        self.y_pos = y_pos;
    }
}
