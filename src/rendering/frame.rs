use serde::{Deserialize, Serialize};

use crate::ppu::palette::SystemPalette;
use crate::rendering::write_tile;
use crate::rom::Rom;

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;
pub const TILE_WIDTH: usize = 8;
pub const TILE_HEIGHT: usize = 8;
pub const SCREEN_WIDTH_IN_TILES: usize = SCREEN_WIDTH / TILE_WIDTH;
pub const SCREEN_HEIGHT_IN_TILES: usize = SCREEN_HEIGHT / TILE_HEIGHT;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Frame {
    pub data: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Frame {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![0; width * height * 3],
            width,
            height,
        }
    }

    pub fn clear(&mut self) {
        self.data = Self::default().data;
    }

    pub fn fill(&mut self, rgb: (u8, u8, u8)) {
        for chunk in self.data.chunks_exact_mut(3) {
            chunk[0] = rgb.0;
            chunk[1] = rgb.1;
            chunk[2] = rgb.2;
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let base = y * 3 * self.width + x * 3;
        if base + 2 < self.data.len() {
            self.data[base] = rgb.0;
            self.data[base + 1] = rgb.1;
            self.data[base + 2] = rgb.2;
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
        let base = y * 3 * self.width + x * 3;
        if base + 2 < self.data.len() {
            (self.data[base], self.data[base + 1], self.data[base + 2])
        } else {
            (0, 0, 0)
        }
    }

    pub fn render_tile(
        &mut self,
        x_pos: usize,
        y_pos: usize,
        rom: &Rom,
        bank: usize,
        tile_n: usize,
        pallet: &[u8; 4],
    ) {
        assert!(bank <= 1);
        let tile = rom.read_tile_chr_rom_bank(bank as u16, tile_n as u16 * 16);
        let system_palette = SystemPalette::new();
        write_tile(self, x_pos, y_pos, tile, &system_palette, pallet);
    }
    pub fn render_tile_from_chr_rom(
        &mut self,
        x_pos: usize,
        y_pos: usize,
        chr_rom: &[u8],
        tile_n: usize,
        pallet: &[u8; 4],
    ) {
        let tile = &chr_rom[(tile_n * 16)..(tile_n * 16 + 16)];
        let system_palette = SystemPalette::new();
        write_tile(self, x_pos, y_pos, tile, &system_palette, pallet);
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new(SCREEN_WIDTH, SCREEN_HEIGHT)
    }
}
