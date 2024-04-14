pub mod addr;
pub mod control;
pub mod mask;
pub mod scroll;
pub mod status;
pub mod sprite;
pub mod palette;

use crate::bus::PollInterrupt;
use crate::ppu::addr::AddressRegister;
use crate::ppu::control::ControlRegister;
use crate::ppu::mask::MaskRegister;
use crate::ppu::palette::SystemPalette;
use crate::ppu::scroll::ScrollRegister;
use crate::ppu::status::StatusRegister;
use crate::rom::Mirroring;

//TODO: 8x16 sprites
pub struct PPU {
    pub chr_rom: Vec<u8>,
    palette_table: [u8; 32],
    pub system_palette: SystemPalette,
    pub vram: [u8; 2048],
    pub oam_addr: u8,
    pub oam_data: [u8; 256],
    pub control_register: ControlRegister,
    mask_register : MaskRegister,
    status_register: StatusRegister,
    scroll_register: ScrollRegister,
    address_register: AddressRegister,
    pub mirroring: Mirroring,
    internal_data_buffer: u8,

    scan_line: u16,
    cycles: usize,

    outstanding_interrupt: bool,
}

impl PollInterrupt for PPU {
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
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring, system_palette: SystemPalette) -> Self {
        PPU {
            chr_rom,
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
            internal_data_buffer: 0,
            scan_line: 0,
            cycles: 0,
            outstanding_interrupt: false
        }
    }

    pub fn tick(&mut self, cycles: u8) -> u16  {
        self.cycles += cycles as usize;
        if self.cycles >= 341 {
            self.cycles -= 341;
            self.scan_line += 1;

            if self.scan_line == 241 {
                self.status_register.set_vertical_blank(true);
                self.status_register.set_sprite_zero_hit(false);
                if self.control_register.generate_nmi() {
                    self.outstanding_interrupt = true;
                }
            } else if self.scan_line >= 262 {
                self.scan_line = 0;
                self.outstanding_interrupt = false;
                self.status_register.set_vertical_blank(false);
                self.status_register.set_sprite_zero_hit(false);
                self.status_register.set_sprite_overflow(false);
            }
        }
        self.scan_line
    }

    fn increment_vram_addr(&mut self) {
        self.address_register.increment(self.control_register.get_vram_increment());
    }

    // Horizontal:
    //   [ A ] [ a ]
    //   [ B ] [ b ]

    // Vertical:
    //   [ A ] [ B ]
    //   [ a ] [ b ]
    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b10_1111_1111_1111; // mirror down 0x3000-0x3eff to 0x2000 - 0x2eff
        let vram_index = mirrored_vram - 0x2000; // to vram vector
        let name_table = vram_index / 0x400; // to name table index
        match (&self.mirroring, name_table) {
            (Mirroring::VERTICAL, 2) |
            (Mirroring::VERTICAL, 3) => vram_index - 0x800,
            (Mirroring::HORIZONTAL, 1) |
            (Mirroring::HORIZONTAL, 2) => vram_index - 0x400,
            (Mirroring::HORIZONTAL, 3) => vram_index - 0x800,
            _ => vram_index
        }
    }

    fn address_to_pattern_table_index(&self, addr: u16) -> u16 {
        (addr - 0x3F00) % 32
    }

    pub fn write_to_data(&mut self, data: u8) {
        let addr = self.address_register.get();
        match addr {
            0x0000..=0x1FFF => print!("Attempt to write to Cartridge ROM space"),
            0x2000..=0x2FFF => self.vram[self.mirror_vram_addr(addr) as usize] = data,
            0x3000..=0x3EFF => print!("address space 0x3000..0x3EFF is not expected to be used, requested = {}", addr),
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => self.palette_table[(addr - 0x10 - 0x3F00) as usize] = data,
            0x3F00..=0x3FFF => self.palette_table[(self.address_to_pattern_table_index(addr)) as usize] = data,
            _               => print!("unexpected access to mirrored space, requested = {}", addr),
        }
        self.address_register.increment(self.control_register.get_vram_increment());
    }

    pub fn read_palette_table(&self, idx: usize) -> u8 {
        let mut palette = self.palette_table[idx];
        if self.mask_register.is_grayscale() {
            palette &= 0x30;
        }
        palette
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.address_register.get();
        self.increment_vram_addr();

        match addr {
            0x0000..=0x1FFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = self.chr_rom[addr as usize];
                result
            },
            0x2000..=0x2FFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            },
            0x3000..=0x3EFF => panic!("address space 0x3000..0x3EFF is not expected to be used, requested = {}", addr),
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C =>  self.read_palette_table((addr - 0x10 - 0x3F00) as usize),
            0x3F00..=0x3FFF => self.read_palette_table((addr - 0x3F00) as usize),
            _               => panic!("unexpected access to mirrored space, requested = {}", addr),
        }
    }

    pub fn write_to_addr(&mut self, value: u8) {
        self.address_register.update(value);
    }

    pub fn write_to_ctrl(&mut self, value: u8) {
        let before_nmi_status = self.control_register.generate_nmi();
        self.control_register.update(value);
        if !before_nmi_status && self.control_register.generate_nmi() && self.status_register.vertical_blank() {
            self.outstanding_interrupt = true;
        }
    }

    pub fn write_to_mask(&mut self, value: u8) {
        self.mask_register.update(value);
    }

    pub fn read_status(&mut self) -> u8 {
        let status = self.status_register.bits();
        self.scroll_register.reset_letch();
        self.address_register.reset_latch();
        self.status_register.set_vertical_blank(false);
        status
    }

    pub fn write_to_scroll(&mut self, value: u8) {
        self.scroll_register.update(value);
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
        self.scroll_register.get_scroll_y()
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
        self.system_palette.get_palette(self.mask_register.get_emphasis_index() as usize)[idx]
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

    pub fn new_empty_rom() -> PPU {
        PPU::new(vec![0; 2048], Mirroring::HORIZONTAL, SystemPalette::new())
    }

    #[test]
    fn test_ppu_vram_writes() {
        let mut ppu = new_empty_rom();
        ppu.write_to_addr(0x23);
        ppu.write_to_addr(0x05);
        ppu.write_to_data(0x66);

        assert_eq!(ppu.vram[0x0305], 0x66);
    }

    #[test]
    fn test_ppu_vram_reads() {
        let mut ppu = new_empty_rom();
        ppu.write_to_ctrl(0);
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_addr(0x23);
        ppu.write_to_addr(0x05);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.address_register.get(), 0x2306);
        assert_eq!(ppu.read_data(), 0x66);
    }

    #[test]
    fn test_ppu_vram_reads_cross_page() {
        let mut ppu = new_empty_rom();
        ppu.write_to_ctrl(0);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x0200] = 0x77;

        ppu.write_to_addr(0x21);
        ppu.write_to_addr(0xff);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.read_data(), 0x66);
        assert_eq!(ppu.read_data(), 0x77);
    }

    #[test]
    fn test_ppu_vram_reads_step_32() {
        let mut ppu = new_empty_rom();
        ppu.write_to_ctrl(0b100);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x01ff + 32] = 0x77;
        ppu.vram[0x01ff + 64] = 0x88;

        ppu.write_to_addr(0x21);
        ppu.write_to_addr(0xff);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.read_data(), 0x66);
        assert_eq!(ppu.read_data(), 0x77);
        assert_eq!(ppu.read_data(), 0x88);
    }

    // Horizontal: https://wiki.nesdev.com/w/index.php/Mirroring
    //   [0x2000 A ] [0x2400 a ]
    //   [0x2800 B ] [0x2C00 b ]
    #[test]
    fn test_vram_horizontal_mirror() {
        let mut ppu = new_empty_rom();
        ppu.write_to_addr(0x24);
        ppu.write_to_addr(0x05);

        ppu.write_to_data(0x66); //write to a

        ppu.write_to_addr(0x28);
        ppu.write_to_addr(0x05);

        ppu.write_to_data(0x77); //write to B

        ppu.write_to_addr(0x20);
        ppu.write_to_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x66); //read from A

        ppu.write_to_addr(0x2C);
        ppu.write_to_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x77); //read from b
    }

    // Vertical: https://wiki.nesdev.com/w/index.php/Mirroring
    //   [0x2000 A ] [0x2400 B ]
    //   [0x2800 a ] [0x2C00 b ]
    #[test]
    fn test_vram_vertical_mirror() {
        let mut ppu = PPU::new(vec![0; 2048], Mirroring::VERTICAL, SystemPalette::new());

        ppu.write_to_addr(0x20);
        ppu.write_to_addr(0x05);

        ppu.write_to_data(0x66); //write to A

        ppu.write_to_addr(0x2C);
        ppu.write_to_addr(0x05);

        ppu.write_to_data(0x77); //write to b

        ppu.write_to_addr(0x28);
        ppu.write_to_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x66); //read from a

        ppu.write_to_addr(0x24);
        ppu.write_to_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x77); //read from B
    }

    #[test]
    fn test_read_status_resets_latch() {
        let mut ppu = new_empty_rom();
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_addr(0x21);
        ppu.write_to_addr(0x23);
        ppu.write_to_addr(0x05);

        ppu.read_data(); //load_into_buffer
        assert_ne!(ppu.read_data(), 0x66);

        ppu.read_status();

        ppu.write_to_addr(0x23);
        ppu.write_to_addr(0x05);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.read_data(), 0x66);
    }

    #[test]
    fn test_ppu_vram_mirroring() {
        let mut ppu = new_empty_rom();
        ppu.write_to_ctrl(0);
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_addr(0x63); //0x6305 -> 0x2305
        ppu.write_to_addr(0x05);

        ppu.read_data(); //load into_buffer
        assert_eq!(ppu.read_data(), 0x66);
        // assert_eq!(ppu.addr.read(), 0x0306)
    }

    #[test]
    fn test_read_status_resets_vblank() {
        let mut ppu = new_empty_rom();
        ppu.status_register.set_vertical_blank(true);

        let status = ppu.read_status();

        assert_eq!(status >> 7, 1);
        assert_eq!(ppu.status_register.bits() >> 7, 0);
    }

    #[test]
    fn test_oam_read_write() {
        let mut ppu = new_empty_rom();
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
        let mut ppu = new_empty_rom();

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