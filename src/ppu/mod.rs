pub mod addr;
pub mod control;
pub mod mask;
pub mod palette;
pub mod scroll;
pub mod sprite;
pub mod status;
mod t_register;

use crate::bus::PollNMI;
use crate::ppu::addr::AddressRegister;
use crate::ppu::control::ControlRegister;
use crate::ppu::mask::MaskRegister;
use crate::ppu::palette::SystemPalette;
use crate::ppu::scroll::ScrollRegister;
use crate::ppu::sprite::Sprite;
use crate::ppu::status::StatusRegister;
use crate::ppu::t_register::TRegister;
use crate::rendering::frame::SCREEN_WIDTH;
use crate::rendering::render_bg;
use crate::rendering::scanline::{Scanline, SpriteColor};
use crate::rom::{Mirroring, Rom};
use std::cmp::{max, min};
use std::ops::Range;

//TODO: 8x16 sprites
pub struct PPU {
    palette_table: [u8; 32],
    system_palette: SystemPalette,
    pub vram: [u8; 2048],
    pub oam_addr: u8,
    pub oam_data: [u8; 256],
    pub control_register: ControlRegister,
    mask_register: MaskRegister,
    status_register: StatusRegister,
    scroll_register: ScrollRegister,
    pub address_register: AddressRegister,
    pub temporary_address_register: TRegister,
    mirroring: Mirroring,
    internal_data_buffer: u8,
    write_toggle: bool,
    sprite_buffer: [Sprite; 8],
    pub sprite_zero_pos: Range<usize>,
    sprite_zero_was_hit_this_frame: bool,

    pub scan_line: i32,
    pub cycles: usize,

    outstanding_interrupt: bool,
    global_cycle: usize,
}

impl PollNMI for PPU {
    fn poll_nmi_status(&mut self) -> bool {
        if self.outstanding_interrupt {
            self.outstanding_interrupt = false;
            true
        } else {
            false
        }
    }
}

impl PPU {
    pub fn new(mirroring: Mirroring, system_palette: SystemPalette) -> Self {
        PPU {
            palette_table: [0; 32],
            system_palette,
            vram: [0; 2048],
            oam_addr: 0,
            oam_data: [0; 256],
            control_register: ControlRegister::new(),
            mask_register: MaskRegister::new(),
            status_register: StatusRegister::new(),
            scroll_register: ScrollRegister::new(),
            mirroring,
            address_register: AddressRegister::new(),
            temporary_address_register: TRegister::new(),
            internal_data_buffer: 0,
            write_toggle: false,
            sprite_buffer: [Sprite::default(); 8],
            sprite_zero_pos: 0..0,
            sprite_zero_was_hit_this_frame: false,
            scan_line: -1,
            cycles: 0,
            outstanding_interrupt: false,
            global_cycle: 0,
        }
    }

    pub fn tick(&mut self, cycles: u8, rom: &Rom, sprite_pixel_buffer: &mut Scanline) -> i32 {
        let pixel_range = self.cycles..(self.cycles + cycles as usize);
        self.cycles += cycles as usize;
        self.global_cycle += cycles as usize;

        // The VBL flag ($2002.7) is cleared by the PPU around 2270 CPU clocks after NMI occurs.
        if self.scan_line == 260 && self.cycles >= 308 {
            self.status_register.set_vertical_blank(false);
            self.status_register.set_sprite_zero_hit(false);
            self.outstanding_interrupt = false;
        }

        if self.cycles >= 341 {
            //println!("Scan line done {} {}", self.scan_line, self.global_cycle);
            if self.show_background() {
                self.address_register
                    .load_x_from(&self.temporary_address_register);
            }

            self.cycles -= 341;
            self.scan_line += 1;

            // Handle visible scanlines
            if self.scan_line < 240 {
                sprite_pixel_buffer.clear();
                self.compute_sprites_next_scanline(rom, sprite_pixel_buffer);
                if self.show_background() {
                    render_bg(self, rom, sprite_pixel_buffer);
                }
            }
            // VBlank start at scanline 241
            else if self.scan_line == 241 {
                self.status_register.set_vertical_blank(true);
                self.status_register.set_sprite_overflow(false);
                if self.control_register.generate_nmi() {
                    self.outstanding_interrupt = true;
                }
            }
            // Reset scanline to -1 after 260 to start pre-render
            else if self.scan_line > 260 {
                self.scan_line = -1;
                self.outstanding_interrupt = false;
                self.sprite_zero_was_hit_this_frame = false;
                if self.show_background() {
                    self.address_register
                        .load_y_from(&self.temporary_address_register);
                }
            }
        }

        // Check for Sprite Zero hit during visible scanlines (0–239)
        if !self.sprite_zero_was_hit_this_frame && self.scan_line >= 0 && self.scan_line < 240 {
            self.check_sprite_zero_hit(pixel_range, sprite_pixel_buffer);
        }

        if self.control_register.get_sprite_size() == 16 {
            println!("Sprite size: 16");
        }

        self.scan_line
    }

    fn check_sprite_zero_hit(&mut self, range_to_check: Range<usize>, scanline: &Scanline) {
        if self.show_sprites()
            && self.scan_line <= 240
            && range_to_check.start < self.sprite_zero_pos.end
            && self.sprite_zero_pos.start < range_to_check.end
            && self.show_background()
            && self.show_sprites()
            && (range_to_check.end > 8 || self.show_sprites_left() && self.show_background_left())
        {
            for pos in max(range_to_check.start, self.sprite_zero_pos.start)
                ..min(range_to_check.end, self.sprite_zero_pos.end)
            {
                if scanline.data[pos].sprite_zero_opaque
                    && !scanline.data[pos].background_color.transparent
                {
                    self.set_sprite_zero_hit();
                    self.sprite_zero_was_hit_this_frame = true;
                    // println!(
                    //     "HIT {} {} {}",
                    //     self.scan_line, self.cycles, self.global_cycle
                    // );
                    break;
                }
            }
        }
    }

    fn compute_sprites_next_scanline(&mut self, rom: &Rom, sprite_pixel_buffer: &mut Scanline) {
        let scan_line = (self.scan_line) as u16;
        let mut current_sprite_slot = 0;
        self.sprite_zero_pos = 0..0;
        for sprite_idx in (0..self.oam_data.len()).step_by(4) {
            let raw = &self.oam_data[sprite_idx..sprite_idx + 4];
            if raw[0] < 239 && scan_line > raw[0] as u16 && scan_line < (raw[0] + 1 + 8) as u16 {
                self.sprite_buffer[current_sprite_slot] =
                    Sprite::new(raw, sprite_idx == 0).unwrap();
                current_sprite_slot += 1;
                if sprite_idx == 0 {
                    let x_start = self.sprite_buffer[current_sprite_slot - 1].get_x();
                    self.sprite_zero_pos = x_start..(x_start + 8);
                }
                if current_sprite_slot >= 8 {
                    self.set_sprite_overflow();
                    break;
                }
            }
        }

        for sprite_idx in (0..current_sprite_slot).rev() {
            let sprite = &self.sprite_buffer[sprite_idx];
            let palette = self.get_sprite_palette(sprite.get_palette_index());
            let bank = self.control_register.get_sprite_pattern_table_address() as usize;
            let tile = rom.read_tile_chr_rom((bank + sprite.get_pattern_index() * 16) as u16);
            let sprite_line = if sprite.is_vertical_flip() {
                7 - (scan_line as usize - sprite.get_y())
            } else {
                scan_line as usize - sprite.get_y()
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

                let x_pos = if sprite.is_horizontal_flip() {
                    sprite.get_x() + 7 - x
                } else {
                    x + sprite.get_x()
                };

                let rgb = self
                    .get_color_from_current_system_palette(palette[color_idx as usize] as usize);

                if x_pos < SCREEN_WIDTH {
                    sprite_pixel_buffer.data[x_pos].sprite_color = SpriteColor {
                        color: rgb,
                        behind_background: !sprite.draw_over_background(),
                        transparent: false,
                    };
                    sprite_pixel_buffer.data[x_pos].sprite_zero_opaque |= sprite.is_sprite_zero();
                }
            }
        }
    }

    fn get_sprite_palette(&self, palette_idx: usize) -> [u8; 4] {
        let start = 0x11 + palette_idx * 4; // + 0x11 is the offset for the sprite palette tables
        [
            0,
            self.read_palette_table(start),
            self.read_palette_table(start + 1),
            self.read_palette_table(start + 2),
        ]
    }

    // Horizontal:
    //   [ A ] [ a ]
    //   [ B ] [ b ]

    // Vertical:
    //   [ A ] [ B ]
    //   [ a ] [ b ]
    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b10_1111_1111_1111; // mirror down 0x3000-0x3eff to 0x2000 - 0x2eff
        let vram_index = mirrored_vram.wrapping_sub(0x2000); // to vram vector
        let name_table = vram_index / 0x400; // to name table index
        match (&self.mirroring, name_table) {
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) => vram_index - 0x800,
            (Mirroring::Horizontal, 1) | (Mirroring::Horizontal, 2) => vram_index - 0x400,
            (Mirroring::Horizontal, 3) => vram_index - 0x800,
            (Mirroring::OneScreenLowerBank, _) => vram_index % 0x400,
            (Mirroring::OneScreenUpperBank, _) => (vram_index % 0x400) + 0x400,
            _ => vram_index,
        }
    }

    fn address_to_pattern_table_index(&self, addr: u16) -> u16 {
        (addr - 0x3F00) % 32
    }

    pub fn write_to_data(&mut self, data: u8, rom: &mut Rom) {
        let addr = self.address_register.data_alt;
        match addr {
            0x0000..=0x1FFF => rom.write_chr_ram(addr, data),
            0x2000..=0x2FFF => self.vram[self.mirror_vram_addr(addr) as usize] = data,
            0x3000..=0x3EFF => self.vram[self.mirror_vram_addr(addr - 0x1000) as usize] = data,
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => {
                self.palette_table[(addr - 0x10 - 0x3F00) as usize] = data
            }
            0x3F00..=0x3FFF => {
                self.palette_table[(self.address_to_pattern_table_index(addr)) as usize] = data
            }
            _ => println!(
                "unexpected access to mirrored space, requested = {:x}",
                self.address_register.data_alt
            ),
        }
        self.address_register
            .increment_alt(self.control_register.get_vram_increment());

        // I am unsure why but using the address register that uses the t register to access the pallet table causes problems.
        // match self.address_register.data_alt {
        //     0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => {
        //         self.palette_table[(self.address_register.data_alt - 0x10 - 0x3F00) as usize] = data
        //     }
        //     0x3F00..=0x3FFF => {
        //         self.palette_table[(self
        //             .address_to_pattern_table_index(self.address_register.data_alt))
        //             as usize] = data
        //     }
        //     _ => (),
        // }
        // self.address_register
        //     .increment_alt(self.control_register.get_vram_increment());
    }

    pub fn read_palette_table(&self, idx: usize) -> u8 {
        let mut palette = self.palette_table[idx];
        if self.mask_register.is_grayscale() {
            palette &= 0x30;
        }
        palette
    }

    pub fn read_data(&mut self, rom: &Rom) -> u8 {
        let addr = self.address_register.data_alt;
        self.address_register
            .increment_alt(self.control_register.get_vram_increment());

        match addr {
            0x0000..=0x1FFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = rom.read_chr_rom(addr);
                result
            }
            0x2000..=0x2FFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..=0x3EFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer =
                    self.vram[self.mirror_vram_addr(addr - 0x1000) as usize];
                result
            }
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => {
                self.internal_data_buffer =
                    self.vram[self.mirror_vram_addr(addr - 0x1000) as usize];
                self.read_palette_table((addr - 0x10 - 0x3F00) as usize)
            }
            0x3F00..=0x3FFF => {
                self.internal_data_buffer =
                    self.vram[self.mirror_vram_addr(addr - 0x1000) as usize];
                self.read_palette_table((addr - 0x3F00) as usize)
            }
            _ => panic!("unexpected access to mirrored space, requested = {}", addr),
        }
    }

    pub fn write_to_addr(&mut self, value: u8) {
        self.temporary_address_register
            .update_addr_write(value, self.write_toggle);
        if self.write_toggle {
            self.address_register
                .update_from(&self.temporary_address_register);
        }

        self.address_register.data_alt = if self.write_toggle {
            (self.address_register.data_alt & 0xFF00) | value as u16
        } else {
            (self.address_register.data_alt & 0x00FF) | ((value as u16) << 8)
        };

        // mirror down address above 0x3FFF
        if self.address_register.data_alt > 0x3FFF {
            self.address_register.data_alt &= 0b11_1111_1111_1111;
        }

        self.write_toggle = !self.write_toggle;
    }

    pub fn write_to_ctrl(&mut self, value: u8) {
        self.temporary_address_register.update_ctrl_write(value);
        let before_nmi_status = self.control_register.generate_nmi();
        self.control_register.update(value);
        if !before_nmi_status
            && self.control_register.generate_nmi()
            && self.status_register.vertical_blank()
        {
            self.outstanding_interrupt = true;
        }
    }

    pub fn write_to_mask(&mut self, value: u8) {
        self.mask_register.update(value);
    }

    pub fn read_status(&mut self) -> u8 {
        let status = self.status_register.bits();
        self.write_toggle = false;
        self.status_register.set_vertical_blank(false);
        status
    }

    pub fn write_to_scroll(&mut self, value: u8) {
        self.temporary_address_register
            .update_scroll_write(value, self.write_toggle);
        self.scroll_register.update(&mut self.write_toggle, value);
    }

    pub fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }

    pub fn read_oam_addr(&mut self) -> u8 {
        self.oam_addr
    }

    pub fn write_to_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    pub fn write_to_oam_data_dma(&mut self, data: &[u8; 256]) {
        for &b in data {
            self.oam_data[self.oam_addr as usize] = b;
            self.oam_addr = self.oam_addr.wrapping_add(1);
        }
    }

    pub fn read_oam_data(&mut self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    pub fn get_scroll_x(&self) -> u8 {
        self.scroll_register.get_scroll_x()
    }

    pub fn get_scroll_y(&self) -> u8 {
        self.address_register.get_tile_y() as u8
    }

    pub fn get_universal_background_color(&self) -> u8 {
        self.read_palette_table(0)
    }

    pub fn show_sprites(&self) -> bool {
        self.mask_register.show_sprites()
    }

    pub fn show_background(&self) -> bool {
        self.mask_register.show_background()
    }

    pub fn show_sprites_left(&self) -> bool {
        self.mask_register.show_sprites_left()
    }

    pub fn show_background_left(&self) -> bool {
        self.mask_register.show_background_left()
    }

    pub fn is_in_vertical_blank(&self) -> bool {
        self.status_register.vertical_blank()
    }

    pub fn get_color_from_current_system_palette(&self, idx: usize) -> (u8, u8, u8) {
        self.system_palette
            .get_palette(self.mask_register.get_emphasis_index() as usize)[idx]
    }

    pub fn set_sprite_zero_hit(&mut self) {
        self.status_register.set_sprite_zero_hit(true);
    }

    pub fn set_sprite_overflow(&mut self) {
        self.status_register.set_sprite_overflow(true);
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub fn new_ppu() -> PPU {
        PPU::new(Mirroring::Horizontal, SystemPalette::new())
    }

    #[test]
    fn test_ppu_vram_writes() {
        let mut rom = Rom::new_blank_test_rom(0);
        let mut ppu = new_ppu();
        ppu.write_to_addr(0x23);
        ppu.write_to_addr(0x05);
        ppu.write_to_data(0x66, &mut rom);

        assert_eq!(ppu.vram[0x0305], 0x66);
    }

    #[test]
    fn test_ppu_vram_reads() {
        let rom = Rom::new_blank_test_rom(0);
        let mut ppu = new_ppu();
        ppu.write_to_ctrl(0);
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_addr(0x23);
        ppu.write_to_addr(0x05);

        ppu.read_data(&rom); //load_into_buffer
        assert_eq!(ppu.address_register.data_alt, 0x2306);
        assert_eq!(ppu.read_data(&rom), 0x66);
    }

    #[test]
    fn test_ppu_vram_reads_cross_page() {
        let rom = Rom::new_blank_test_rom(0);
        let mut ppu = new_ppu();
        ppu.write_to_ctrl(0);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x0200] = 0x77;

        ppu.write_to_addr(0x21);
        ppu.write_to_addr(0xff);

        ppu.read_data(&rom); //load_into_buffer
        assert_eq!(ppu.read_data(&rom), 0x66);
        assert_eq!(ppu.read_data(&rom), 0x77);
    }

    #[test]
    fn test_ppu_vram_reads_step_32() {
        let rom = Rom::new_blank_test_rom(0);
        let mut ppu = new_ppu();
        ppu.write_to_ctrl(0b100);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x01ff + 32] = 0x77;
        ppu.vram[0x01ff + 64] = 0x88;

        ppu.write_to_addr(0x21);
        ppu.write_to_addr(0xff);

        ppu.read_data(&rom); //load_into_buffer
        assert_eq!(ppu.read_data(&rom), 0x66);
        assert_eq!(ppu.read_data(&rom), 0x77);
        assert_eq!(ppu.read_data(&rom), 0x88);
    }

    // Horizontal: https://wiki.nesdev.com/w/index.php/Mirroring
    //   [0x2000 A ] [0x2400 a ]
    //   [0x2800 B ] [0x2C00 b ]
    #[test]
    fn test_vram_horizontal_mirror() {
        let mut rom = Rom::new_blank_test_rom(0);
        let mut ppu = new_ppu();
        ppu.write_to_addr(0x24);
        ppu.write_to_addr(0x05);

        ppu.write_to_data(0x66, &mut rom); //write to, a

        ppu.write_to_addr(0x28);
        ppu.write_to_addr(0x05);

        ppu.write_to_data(0x77, &mut rom); //write to, B

        ppu.write_to_addr(0x20);
        ppu.write_to_addr(0x05);

        ppu.read_data(&rom); //load into buffer
        assert_eq!(ppu.read_data(&rom), 0x66); //read from A

        ppu.write_to_addr(0x2C);
        ppu.write_to_addr(0x05);

        ppu.read_data(&rom); //load into buffer
        assert_eq!(ppu.read_data(&rom), 0x77); //read from b
    }

    // Vertical: https://wiki.nesdev.com/w/index.php/Mirroring
    //   [0x2000 A ] [0x2400 B ]
    //   [0x2800 a ] [0x2C00 b ]
    #[test]
    fn test_vram_vertical_mirror() {
        let mut rom = Rom::new_blank_test_rom(0);
        let mut ppu = PPU::new(Mirroring::Vertical, SystemPalette::new());

        ppu.write_to_addr(0x20);
        ppu.write_to_addr(0x05);

        ppu.write_to_data(0x66, &mut rom); //write to, A

        ppu.write_to_addr(0x2C);
        ppu.write_to_addr(0x05);

        ppu.write_to_data(0x77, &mut rom); //write to, b

        ppu.write_to_addr(0x28);
        ppu.write_to_addr(0x05);

        ppu.read_data(&rom); //load into buffer
        assert_eq!(ppu.read_data(&rom), 0x66); //read from a

        ppu.write_to_addr(0x24);
        ppu.write_to_addr(0x05);

        ppu.read_data(&rom); //load into buffer
        assert_eq!(ppu.read_data(&rom), 0x77); //read from B
    }

    #[test]
    fn test_read_status_resets_latch() {
        let rom = Rom::new_blank_test_rom(0);
        let mut ppu = new_ppu();
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_addr(0x21);
        ppu.write_to_addr(0x23);
        ppu.write_to_addr(0x05);

        ppu.read_data(&rom); //load_into_buffer
        assert_ne!(ppu.read_data(&rom), 0x66);

        ppu.read_status();

        ppu.write_to_addr(0x23);
        ppu.write_to_addr(0x05);

        ppu.read_data(&rom); //load_into_buffer
        assert_eq!(ppu.read_data(&rom), 0x66);
    }

    #[test]
    fn test_ppu_vram_mirroring() {
        let rom = Rom::new_blank_test_rom(0);
        let mut ppu = new_ppu();
        ppu.write_to_ctrl(0);
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_addr(0x63); //0x6305 -> 0x2305
        ppu.write_to_addr(0x05);

        ppu.read_data(&rom); //load into_buffer
        assert_eq!(ppu.read_data(&rom), 0x66);
        // assert_eq!(ppu.addr.read(), 0x0306)
    }

    #[test]
    fn test_read_status_resets_vblank() {
        let mut ppu = new_ppu();
        ppu.status_register.set_vertical_blank(true);

        let status = ppu.read_status();

        assert_eq!(status >> 7, 1);
        assert_eq!(ppu.status_register.bits() >> 7, 0);
    }

    #[test]
    fn test_oam_read_write() {
        let mut ppu = new_ppu();
        ppu.write_to_oam_addr(0x10);
        ppu.write_to_oam_data(0x66);
        ppu.write_to_oam_data(0x77);

        ppu.write_to_oam_addr(0x10);
        assert_eq!(ppu.read_oam_data(), 0x66);

        ppu.write_to_oam_addr(0x11);
        assert_eq!(ppu.read_oam_data(), 0x77);
    }

    #[test]
    fn test_oam_dma() {
        let mut ppu = new_ppu();

        let mut data = [0x66; 256];
        data[0] = 0x77;
        data[255] = 0x88;

        ppu.write_to_oam_addr(0x10);
        ppu.write_to_oam_data_dma(&data);

        ppu.write_to_oam_addr(0xf); //wrap around
        assert_eq!(ppu.read_oam_data(), 0x88);

        ppu.write_to_oam_addr(0x10);
        ppu.write_to_oam_addr(0x77);
        ppu.write_to_oam_addr(0x11);
        ppu.write_to_oam_addr(0x66);
    }
}
