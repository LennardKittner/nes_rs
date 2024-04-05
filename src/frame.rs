use crate::render::write_tile;

pub struct Frame {
    pub data: Vec<u8>
}

impl Frame {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 240;

    pub fn new() -> Self {
        Self {
            data: vec![0; Self::WIDTH * Self::HEIGHT * 3],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let base = y * 3 * Self::WIDTH + x * 3;
        if base + 2 < self.data.len() {
            self.data[base] = rgb.0;
            self.data[base + 1] = rgb.1;
            self.data[base + 2] = rgb.2;
        }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_tile(frame: &mut Frame, x_pos: usize, y_pos:usize, chr_rom: &[u8], bank: usize, tile_n: usize) {
    assert!(bank <= 1);
    let bank_address = bank * 0x1000;

    let tile = &chr_rom[(bank_address + tile_n * 16)..(bank_address + tile_n * 16 + 16)];

    write_tile(frame, x_pos, y_pos, tile, &[0x01, 0x23, 0x27, 0x30]);
}
