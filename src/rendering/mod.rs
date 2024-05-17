pub mod frame;
mod rect;
pub mod scanline;

use std::ops::Neg;
use itertools::Itertools;
use crate::ppu::palette::SystemPalette;
use crate::ppu::PPU;
use crate::ppu::sprite::Sprite;
use crate::rendering::frame::Frame;
use crate::rendering::rect::Rect;
use crate::rendering::scanline::{BackgroundColor, Scanline, SpriteColor};
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

pub fn write_tile(frame: &mut Frame, x_pos: usize, y_pos: usize, tile: &[u8], palette: &[u8; 4]) {
    for y in 0..8 {
        let mut upper = tile[y];
        let mut lower = tile[y + 8];

        for x in (0..8).rev() {
            let color_idx = (1 & lower) << 1 | (1 & upper);
            upper >>= 1;
            lower >>= 1;

            let rgb = SystemPalette::new().get_palette(0)[palette[color_idx as usize] as usize];
            frame.set_pixel(x_pos + x, y_pos + y, rgb);
        }
    }
}

pub fn render<'a>(ppu: &mut PPU, scanline: &mut Scanline, scanline_pos: usize) {
    if ppu.show_background() {
        render_background_current_scanline(ppu, scanline);
    }
    if ppu.show_sprites() {
        render_sprites(ppu, scanline, scanline_pos);
    }
}

pub fn render_sprites(ppu: &mut PPU, scanline: &mut Scanline, scanline_pos: usize) {
    let scanline_pos = scanline_pos as u8;
    let sprites = (0..ppu.oam_data.len()).step_by(4).filter_map(|idx| {
        let raw = &ppu.oam_data[idx..idx+4];
        if raw[0] >= 239 || scanline_pos < raw[0] + 1 || scanline_pos >= (raw[0] + 1 + 8) {
            None
        } else {
            Sprite::new(raw, idx == 0)
        }
    }).collect_vec();

    if sprites.len() > 8 {
        ppu.set_sprite_overflow();
    }

    for sprite in sprites.iter().take(8).rev() {
        let palette = get_sprite_palette(ppu, sprite.get_palette_index());
        let bank = ppu.control_register.get_sprite_pattern_table_address() as usize;
        let tile = ppu.chr_rom[(bank + sprite.get_pattern_index() * 16)..(bank + sprite.get_pattern_index() * 16 + 16)].to_vec();
        let sprite_line = if sprite.is_vertical_flip() {
            7 - (scanline_pos as usize - sprite.get_y())
        } else {
            scanline_pos as usize - sprite.get_y()
        };

        let mut upper = tile[sprite_line];
        let mut lowwer = tile[sprite_line + 8];

        for x in (0..8).rev() {
            let color_idx = (1 & lowwer) << 1 | (1 & upper);
            upper >>= 1;
            lowwer >>= 1;

            if color_idx == 0 {
                continue;
            }

            if sprite.is_sprite_zero() {
                ppu.set_sprite_zero_hit();
            }

            let rgb = ppu.get_color_from_current_system_palette(palette[color_idx as usize] as usize);
            let x_pos = if sprite.is_horizontal_flip() {
                sprite.get_x() + 7 - x
            } else {
                x + sprite.get_x()
            };
            if x_pos < Frame::WIDTH {
                scanline.data[x_pos].sprite_color = SpriteColor {
                    color: rgb,
                    behind_background: !sprite.draw_over_background(),
                    transparent: false,
                };
            }
        }
    }
}

pub fn render_background_current_scanline(ppu: &mut PPU, scanline: &mut Scanline) {
    render_bg(ppu, scanline);
}

//TODO: maybe extract rendering loop
//TODO: remove nametable bits form status
fn render_bg(ppu: &mut PPU, scanline: &mut Scanline) {
    let (main_name_table, second_name_table) = match (&ppu.mirroring, ppu.address_register.get_name_table()) {
        (Mirroring::VERTICAL, 0b00) | (Mirroring::VERTICAL, 0b10) | (Mirroring::HORIZONTAL, 0b00) | (Mirroring::HORIZONTAL, 0b01) => {
            (&ppu.vram[0..0x400], &ppu.vram[0x400..0x800])
        },
        (Mirroring::VERTICAL, 0b01) | (Mirroring::VERTICAL, 0b11) | (Mirroring::HORIZONTAL, 0b10) | (Mirroring::HORIZONTAL, 0b11) => {
            (&ppu.vram[0x400..0x800], &ppu.vram[0..0x400])
        },
        (_, _) => panic!("Unsupported mirroring mode: {:?}", ppu.mirroring),
    };

    let bank = ppu.control_register.get_background_pattern_table_address();
    let attribute_table = &main_name_table[0x3C0..0x400];
    let line = ppu.address_register.get_inner_tile_y_offset();

    let tile_x = ppu.address_register.get_tile_x();
    let tile_y = ppu.address_register.get_tile_y();
    
    let shift_x = ppu.get_scroll_x() as usize;

    for tile_x in tile_x..32 {
        let tile_idx = main_name_table[32 * tile_y + tile_x] as u16;

        let tile = &ppu.chr_rom[(bank + tile_idx * 16) as usize..(bank + tile_idx * 16 + 16) as usize];
        if !ppu.show_background_left() && tile_x == 0 {
            continue;
        }

        let palette = get_bg_palette(ppu, attribute_table, tile_x, tile_y);
        let mut upper = tile[line];
        let mut lower = tile[line + 8];

        for x in (0..8).rev() {
            let color_idx = (1 & lower) << 1 | (1 & upper);
            upper >>= 1;
            lower >>= 1;

            let pixel_x = tile_x * 8 + x;

            if pixel_x > shift_x {
                let rgb = ppu.get_color_from_current_system_palette(palette[color_idx as usize] as usize);
                scanline.data[pixel_x - shift_x].background_color = BackgroundColor {
                    color: rgb,
                    transparent: color_idx == 0,
                };
            }
        }
    }
    

    ppu.address_register.vertical_name_table_overflow();
    let attribute_table = &second_name_table[0x3C0..0x400];
    for tile_x in 0..(tile_x+1) {
        let tile_idx = second_name_table[32 * tile_y + tile_x] as u16;

        let tile = &ppu.chr_rom[(bank + tile_idx * 16) as usize..(bank + tile_idx * 16 + 16) as usize];
        if !ppu.show_background_left() && tile_x == 0 {
            continue;
        }

        let palette = get_bg_palette(ppu, attribute_table, tile_x, tile_y);
        let mut upper = tile[line];
        let mut lower = tile[line + 8];

        for x in (0..8).rev() {
            let color_idx = (1 & lower) << 1 | (1 & upper);
            upper >>= 1;
            lower >>= 1;

            let pixel_x = tile_x * 8 + x;

            if pixel_x <= shift_x && pixel_x + (256 - shift_x) < 256 {
                let rgb = ppu.get_color_from_current_system_palette(palette[color_idx as usize] as usize);
                scanline.data[pixel_x + (256 - shift_x)].background_color = BackgroundColor {
                    color: rgb,
                    transparent: color_idx == 0,
                };
            }
        }
    }
    ppu.address_register.vertical_name_table_overflow();
    
    if line == 7 {
        ppu.address_register.set_tile_y((tile_y+1) as u8);
        ppu.address_register.horizontal_name_table_overflow();
    }
    ppu.address_register.set_inner_tile_y_offset((line+1) as u8);
}

fn render_name_table(ppu: &PPU, scanline: &mut Scanline, name_table: &[u8], view_port: Rect, shift_x: isize, shift_y: isize, scanline_pos: usize) {
    let bank = ppu.control_register.get_background_pattern_table_address();
    let attribute_table = &name_table[0x3C0..0x400];

    let shifted_scanline_pos = scanline_pos.wrapping_add_signed(shift_y.neg()) % 256;

    for i in (shifted_scanline_pos / 8 * 32)..(shifted_scanline_pos / 8 * 32 + 32) {
        let tile_idx = name_table[i] as u16;
        let tile_x = i % 32;
        let tile_y = i / 32;
        let line = shifted_scanline_pos % 8;
        let tile = &ppu.chr_rom[(bank + tile_idx * 16) as usize..(bank + tile_idx * 16 + 16) as usize];
        if !ppu.show_background_left() && tile_x == 0 {
            continue;
        }

        let palette = get_bg_palette(ppu, attribute_table, tile_x, tile_y);
        let mut upper = tile[line];
        let mut lower = tile[line + 8];

        for x in (0..8).rev() {
            let color_idx = (1 & lower) << 1 | (1 & upper);
            upper >>= 1;
            lower >>= 1;

            let pixel_x = tile_x * 8 + x;
            let pixel_y = tile_y * 8 + line;

            if pixel_x >= view_port.x1 && pixel_x < view_port.x2 && pixel_y >= view_port.y1 && pixel_y < view_port.y2 {
                let rgb = ppu.get_color_from_current_system_palette(palette[color_idx as usize] as usize);
                scanline.data[(shift_x + pixel_x as isize) as usize].background_color = BackgroundColor {
                    color: rgb,
                    transparent: color_idx == 0,
                };
            }
        }
    }
}
