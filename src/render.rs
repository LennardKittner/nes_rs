use crate::ppu::{PPU, SYSTEM_PALLET};
use crate::frame::Frame;
use crate::ppu::sprite::Sprite;


pub fn get_bg_palette(ppu: &PPU, x_pos: usize, y_pos: usize) -> [u8; 4] {
    let attribute_table_idx = y_pos / 4 * 8 + x_pos / 4;
    let attribute = ppu.vram[0x3C0 + attribute_table_idx]; //TODO: nametable

    let palette_idx = match (x_pos % 4 / 2, y_pos % 4 / 2) {
        (0, 0) => attribute & 0b11,
        (1, 0) => (attribute >> 2) & 0b11,
        (0, 1) => (attribute >> 4) & 0b11,
        (1, 1) => (attribute >> 6) & 0b11,
        (_, _) => panic!("Impossible tile index"),
    };

    let palette_start = (1 + palette_idx * 4) as usize;
    [ppu.palette_table[0], ppu.palette_table[palette_start], ppu.palette_table[palette_start+1], ppu.palette_table[palette_start+2]]
}

pub fn get_sprite_palette(ppu: &PPU, palette_idx: usize) -> [u8; 4] {
    let start = 0x11 + palette_idx * 4; //TODO: why 0x11
    [
        0,
        ppu.palette_table[start],
        ppu.palette_table[start + 1],
        ppu.palette_table[start + 2],
    ]
}

pub fn write_tile(frame: &mut Frame, x_pos: usize, y_pos: usize, tile: &[u8], palette: &[u8; 4]) {
    for y in 0..8 {
        let mut upper = tile[y];
        let mut lower = tile[y + 8];

        for x in (0..8).rev() {
            let color_idx = (1 & lower) << 1 | (1 & upper);
            upper >>= 1;
            lower >>= 1;

            let rgb = SYSTEM_PALLET[palette[color_idx as usize] as usize];
            frame.set_pixel(x_pos + x, y_pos + y, rgb);
        }
    }
}

pub fn render(ppu: &PPU, frame: &mut Frame) {
    render_background(ppu, frame);
    render_sprites(ppu, frame);
}

//TODO: priority
fn render_sprites(ppu: &PPU, frame: &mut Frame) {
    let sprites = (0..ppu.oam_data.len()).step_by(4).rev().map(|idx| {
        let raw = &ppu.oam_data[idx..idx+4];
        Sprite::new(raw).unwrap()
    });
    for sprite in sprites {
        let palette = get_sprite_palette(ppu, sprite.get_palette_index());
        let bank = ppu.control_register.get_sprite_pattern_table_address() as usize;
        let tile = &ppu.chr_rom[(bank + sprite.get_pattern_index() * 16)..(bank + sprite.get_pattern_index() * 16 + 16)];
        
        for y in 0..8 {
            let mut upper = tile[y];
            let mut lowwer = tile[y + 8];
            for x in (0..8).rev() {
                let color_idx = (1 & lowwer) << 1 | (1 & upper);
                upper >>= 1;
                lowwer >>= 1;
                if color_idx == 0 {
                    continue;
                }
                let rgb = SYSTEM_PALLET[palette[color_idx as usize] as usize];
                match (sprite.is_horizontal_flip(), sprite.is_vertical_flip()) {
                    (false, false) => frame.set_pixel(x + sprite.get_x(), y + sprite.get_y(), rgb),
                    (true, false) => frame.set_pixel(sprite.get_x() + 7 - x, y + sprite.get_y(), rgb),
                    (false, true) => frame.set_pixel(x + sprite.get_x(), sprite.get_y() + 7 - y, rgb),
                    (true, true) => frame.set_pixel(sprite.get_x() + 7 - x, sprite.get_y() + 7 - y, rgb),
                }
            }
        }
    }
}

fn render_background(ppu: &PPU, frame: &mut Frame) {
    let bank = ppu.control_register.get_background_pattern_table_address();
    for i in 0..0x03C0 { //TODO: nametable
        let tile_idx = ppu.vram[i] as u16;
        let tile_x = i % 32;
        let tile_y = i / 32;
        let tile = &ppu.chr_rom[(bank + tile_idx * 16) as usize..(bank + tile_idx * 16 + 16) as usize];
        write_tile(frame, tile_x * 8, tile_y * 8, tile, &get_bg_palette(ppu, tile_x, tile_y));
    }
}