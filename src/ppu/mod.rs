pub mod addr;
pub mod control;
pub mod mask;
pub mod scroll;
pub mod status;
pub mod sprite;

use crate::bus::PollInterrupt;
use crate::ppu::addr::AddressRegister;
use crate::ppu::control::ControlRegister;
use crate::ppu::mask::MaskRegister;
use crate::ppu::scroll::ScrollRegister;
use crate::ppu::status::StatusRegister;
use crate::rom::Mirroring;

#[rustfmt::skip]
pub static SYSTEM_PALLET: [(u8, u8, u8); 64] = [
    (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E),
    (0xC7, 0x00, 0x28), (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00),
    (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05),
    (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF), (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA),
    (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00), (0xC4, 0x62, 0x00),
    (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21),
    (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF),
    (0xD4, 0x80, 0xFF), (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12),
    (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E), (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF),
    (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D), (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF),
    (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3), (0xFF, 0xD2, 0xB0),
    (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA),
    (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11)
];

pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    oam_addr: u8,
    pub oam_data: [u8; 256],
    pub control_register: ControlRegister,
    mask_register : MaskRegister,
    status_register: StatusRegister,
    scroll_register: ScrollRegister,
    address_register: AddressRegister,
    mirroring: Mirroring,
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
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        PPU {
            chr_rom,
            palette_table: [0; 32],
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

    pub(crate) fn tick(&mut self, cycles: u8) -> bool  {
        self.cycles += cycles as usize;
        if self.cycles >= 341 {
            self.cycles -= 341;
            self.scan_line += 1;

            //TODO: sprite zero hit?
            if self.scan_line == 241 {
                self.status_register.set_vertical_blank(true);
                if self.control_register.generate_nmi() {
                    self.outstanding_interrupt = true;
                }
            } else if self.scan_line >= 262 {
                self.scan_line = 0;
                self.outstanding_interrupt = false;
                self.status_register.set_vertical_blank(false);
                return true;
            }
        }
        false
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

    pub fn write_to_data(&mut self, data: u8) {
        let addr = self.address_register.get();
        match addr {
            0x0000..=0x1FFF => panic!("Attempt to write to Cartridge ROM space"),
            0x2000..=0x2FFF => self.vram[self.mirror_vram_addr(addr) as usize] = data,
            0x3000..=0x3EFF => panic!("address space 0x3000..0x3EFF is not expected to be used, requested = {}", addr),
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => self.palette_table[(addr - 0x10 - 0x3F00) as usize] = data,
            0x3F00..=0x3FFF => self.palette_table[(addr - 0x3F00) as usize] = data,
            _               => panic!("unexpected access to mirrored space, requested = {}", addr),
        }
        self.address_register.increment(self.control_register.get_vram_increment());
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
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => self.palette_table[(addr - 0x10 - 0x3F00) as usize],
            0x3F00..=0x3FFF => self.palette_table[(addr - 0x3F00) as usize],
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
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub fn new_empty_rom() -> PPU {
        PPU::new(vec![0; 2048], Mirroring::HORIZONTAL)
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
        let mut ppu = PPU::new(vec![0; 2048], Mirroring::VERTICAL);

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