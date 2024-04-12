pub mod frame;
mod rect;

use crate::ppu::pallet::Pallet;
use crate::ppu::PPU;
use crate::ppu::sprite::Sprite;
use crate::rendering::frame::Frame;
use crate::rendering::rect::Rect;
use crate::rom::Mirroring;

pub fn get_bg_palette(ppu: &PPU, attribute_table: &[u8], x_pos: usize, y_pos: usize) -> [u8; 4] {
    let attribute_table_idx = y_pos / 4 * 8 + x_pos / 4;
    let attribute = attribute_table[attribute_table_idx];

    let palette_idx = match (x_pos % 4 / 2, y_pos % 4 / 2) {
        (0, 0) => attribute & 0b11,
        (1, 0) => (attribute >> 2) & 0b11,
        (0, 1) => (attribute >> 4) & 0b11,
        (1, 1) => (attribute >> 6) & 0b11,
        (_, _) => panic!("Impossible tile index"),
    };

    let palette_start = (1 + palette_idx * 4) as usize;
    [ppu.get_universal_background_color(), ppu.read_palette_table(palette_start), ppu.read_palette_table(palette_start+1), ppu.read_palette_table(palette_start+2)]
}

pub fn get_sprite_palette(ppu: &PPU, palette_idx: usize) -> [u8; 4] {
    let start = 0x11 + palette_idx * 4; // + 0x11 is the offset for the sprite palette tables
    [
        0,
        ppu.read_palette_table(start),
        ppu.read_palette_table(start + 1),
        ppu.read_palette_table(start + 2),
    ]
}

pub fn write_tile(ppu: &PPU, frame: &mut Frame, x_pos: usize, y_pos: usize, tile: &[u8], palette: &[u8; 4]) {
    let view_port = Rect::new(0, 0, usize::MAX, usize::MAX);
    write_tile_in_view_port(ppu, frame, x_pos, y_pos, 0, 0, &view_port, tile, palette);
}

pub fn write_tile_in_view_port(ppu: &PPU, frame: &mut Frame, x_pos: usize, y_pos: usize, shift_x: isize, shift_y: isize, view_port: &Rect, tile: &[u8], palette: &[u8; 4]) {
    for y in 0..8 {
        let mut upper = tile[y];
        let mut lower = tile[y + 8];

        for x in (0..8).rev() {
            let color_idx = (1 & lower) << 1 | (1 & upper);
            upper >>= 1;
            lower >>= 1;

            let pixel_x = x_pos + x;
            let pixel_y = y_pos + y;

            if pixel_x >= view_port.x1 && pixel_x < view_port.x2 && pixel_y >= view_port.y1 && pixel_y < view_port.y2 {
                let rgb = ppu.get_color_from_current_system_pallet(palette[color_idx as usize] as usize);
                frame.set_pixel((shift_x + pixel_x as isize) as usize, (shift_y + pixel_y as isize) as usize, rgb);
            }
        }
    }
}

pub fn render(ppu: &PPU, frame: &mut Frame, scanline: usize) {
    if ppu.show_background() {
        render_background(ppu, frame, scanline);
    }
    if ppu.show_sprites() {
        render_sprites(ppu, frame);
    }
}

//TODO: more precise drawing
// real NES: At any given pixel, if the frontmost opaque sprite's priority bit is true (1), an opaque background pixel is drawn in front of it.
// Emulation: At any given pixel, only if all opaque sprite's priority bits are true (1), an opaque background pixel is drawn in front of them.
pub fn render_sprites(ppu: &PPU, frame: &mut Frame) {
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

                let rgb = ppu.get_color_from_current_system_pallet(palette[color_idx as usize] as usize);
                match (sprite.is_horizontal_flip(), sprite.is_vertical_flip()) {
                    (false, false) => draw_sprite_pixel(frame, ppu, x + sprite.get_x(), y + sprite.get_y(), sprite.draw_over_background(), rgb),
                    (true, false) => draw_sprite_pixel(frame, ppu, sprite.get_x() + 7 - x, y + sprite.get_y(), sprite.draw_over_background(), rgb),
                    (false, true) => draw_sprite_pixel(frame, ppu, x + sprite.get_x(), sprite.get_y() + 7 - y, sprite.draw_over_background(), rgb),
                    (true, true) => draw_sprite_pixel(frame, ppu, sprite.get_x() + 7 - x, sprite.get_y() + 7 - y, sprite.draw_over_background(), rgb),
                }
            }
        }
    }
}

fn draw_sprite_pixel(frame: &mut Frame, ppu: &PPU, x: usize, y: usize, draw_over_background: bool, color: (u8, u8, u8)) {
    if !draw_over_background && frame.get_pixel(x, y) != ppu.get_color_from_current_system_pallet(ppu.get_universal_background_color() as usize)
        || (!ppu.show_sprites_left() && x < 8) {
        return;
    }
    frame.set_pixel(x, y, color);
}

pub fn render_background(ppu: &PPU, frame: &mut Frame, scanline: usize) {
    let scroll_x = ppu.get_scroll_x() as usize;
    let scroll_y = ppu.get_scroll_y() as usize;

    let (main_name_table, second_name_table) = match (&ppu.mirroring, ppu.control_register.get_nametable_base()) {
        (Mirroring::VERTICAL, 0x2000) | (Mirroring::VERTICAL, 0x2800) | (Mirroring::HORIZONTAL, 0x2000) | (Mirroring::HORIZONTAL, 0x2400) => (&ppu.vram[0..0x400], &ppu.vram[0x400..0x800]),
        (Mirroring::VERTICAL, 0x2400) | (Mirroring::VERTICAL, 0x2C00) | (Mirroring::HORIZONTAL, 0x2800) | (Mirroring::HORIZONTAL, 0x2C00) => (&ppu.vram[0x400..0x800], &ppu.vram[0..0x400]),
        (_, _) => panic!("Unsupported mirroring mode: {:?}", ppu.mirroring),
    };
    
    render_name_table(ppu, frame, main_name_table, Rect::new(scroll_x, scroll_y, 256, 240), -(scroll_x as isize), -(scroll_y as isize), scanline, true);
    if ppu.get_scroll_x() > 0 {
        render_name_table(ppu, frame, second_name_table, Rect::new(0, 0, scroll_x, 240), (256 - scroll_x) as isize, 0, scanline, false);
    }
    if ppu.get_scroll_y() > 0 {
        render_name_table(ppu, frame, second_name_table, Rect::new(0, 0, 256, scroll_y), 0, (240 - scroll_y) as isize, scanline, false);
    }
}

fn render_name_table(ppu: &PPU, frame: &mut Frame, name_table: &[u8], view_port: Rect, shift_x: isize, shift_y: isize, scanline: usize, p: bool) {
    let bank = ppu.control_register.get_background_pattern_table_address();
    let attribute_table = &name_table[0x3C0..0x400];
    
    for i in (scanline / 8 * 32)..(scanline / 8 * 32 + 32) {
        let tile_idx = name_table[i] as u16;
        let tile_x = i % 32;
        let tile_y = i / 32;
        let tile = &ppu.chr_rom[(bank + tile_idx * 16) as usize..(bank + tile_idx * 16 + 16) as usize];
        if !ppu.show_background_left() && tile_x == 0 {
            continue;
        }
        
        let t = if p {
            [0, 10, 20, 30]
        } else {
            get_bg_palette(ppu, attribute_table, tile_x, tile_y)
        };
        
        write_tile_in_view_port(ppu, frame, tile_x * 8, tile_y * 8, shift_x, shift_y, &view_port, tile, &t);
    }
}
