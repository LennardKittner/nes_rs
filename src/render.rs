use crate::ppu::{PPU, SYSTEM_PALLET};
use crate::frame::Frame;


pub fn write_tile(frame: &mut Frame, x_pos: usize, y_pos: usize, tile: &[u8]) {
    for y in 0..8 {
        let mut upper = tile[y];
        let mut lower = tile[y + 8];

        for x in (0..8).rev() {
            let value = (1 & upper) << 1 | (1 & lower);
            upper >>= 1;
            lower >>= 1;

            let rgb = match value {
                0 => SYSTEM_PALLET[0x01],
                1 => SYSTEM_PALLET[0x23],
                2 => SYSTEM_PALLET[0x27],
                3 => SYSTEM_PALLET[0x30],
                _ => panic!("Pixel color out of range")
            };
            frame.set_pixel(x_pos + x, y_pos + y, rgb);
        }
    }
}

pub fn render(ppu: &PPU, frame: &mut Frame) {
    let bank = ppu.control_register.get_background_pattern_table_address();

    for i in 0..0x03C0 { //TODO: nametable
        let tile_idx = ppu.vram[i] as u16;
        let tile_x = i % 32;
        let tile_y = i / 32;
        let tile = &ppu.chr_rom[(bank + tile_idx * 16) as usize..(bank + tile_idx * 16 + 16) as usize];
        write_tile(frame, tile_x*8, tile_y*8, tile);
    }
}