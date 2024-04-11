
#[derive(Debug)]
pub struct Sprite {
    y_pos: u8,
    pattern_index: u8,
    attributes: u8,
    x_pos: u8,
}

impl Sprite {
    pub fn new(raw: &[u8]) -> Option<Self> {
        if raw.len() < 4 {
            None
        } else {
            Some(Sprite {
                y_pos: raw[0],
                pattern_index: raw[1],
                attributes: raw[2],
                x_pos: raw[3],
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
}