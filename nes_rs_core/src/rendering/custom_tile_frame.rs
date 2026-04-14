use crate::{rendering::frame::Frame, rom::Rom};

pub struct CustomTileFrame {
    pub frame: Frame,
    pub tile_map: Vec<u8>,
    pub attribute_table: Vec<[u8; 4]>,
}

impl CustomTileFrame {
    pub fn new(
        width: usize,
        height: usize,
        tile_map: Vec<u8>,
        attribute_table: Vec<[u8; 4]>,
    ) -> Self {
        Self {
            frame: Frame::new(width, height),
            tile_map,
            attribute_table,
        }
    }

    pub fn update(&mut self, rom: &Rom, bank: usize) {
        for (i, &tile_idx) in self.tile_map.iter().enumerate() {
            let x_in_tiles = i % (self.frame.width / 8);
            let y_in_tiles = i / (self.frame.width / 8);
            let palette = self.attribute_table
                [x_in_tiles / 2 + (self.frame.width / (8 * 2)) * (y_in_tiles / 2)];
            let x = x_in_tiles * 8;
            let y = y_in_tiles * 8;
            self.frame
                .render_tile(x, y, rom, bank, tile_idx as usize, &palette);
        }
    }
}
