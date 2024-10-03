use crate::rendering::frame::Frame;
use crate::rom::Rom;

pub struct FPSFrame {
    pub frame: Frame,
    numbers_offset: usize,
    letters_offset: usize,
    palette: [u8; 4],
}

impl FPSFrame {
    pub fn new(numbers_offset: usize, letters_offset: usize, palette: [u8; 4]) -> Self {
        Self {
            frame: Frame::new(48, 8),
            numbers_offset,
            letters_offset,
            palette,
        }
    }

    pub fn update(&mut self, rom: &Rom, bank: usize, fps: usize, background_color_idx: u8) {
        let hundreds = fps / 100;
        let tenth = (fps % 100) / 10;
        let ones = fps % 10;
        self.palette[0] = background_color_idx;
        self.frame.render_tile(
            0,
            0,
            rom,
            bank,
            self.numbers_offset + hundreds,
            &self.palette,
        );
        self.frame
            .render_tile(8, 0, rom, bank, self.numbers_offset + tenth, &self.palette);
        self.frame
            .render_tile(16, 0, rom, bank, self.numbers_offset + ones, &self.palette);
        self.frame
            .render_tile(24, 0, rom, bank, self.letters_offset + 5, &self.palette);
        self.frame
            .render_tile(32, 0, rom, bank, self.letters_offset + 15, &self.palette);
        self.frame
            .render_tile(40, 0, rom, bank, self.letters_offset + 18, &self.palette);
    }
}
