pub mod addr;
pub mod control;
pub mod mask;
pub mod scroll;
pub mod status;

use crate::bus::Interrupt;
use crate::ppu::addr::AddressRegister;
use crate::ppu::control::ControlRegister;
use crate::ppu::mask::MaskRegister;
use crate::ppu::scroll::ScrollRegister;
use crate::ppu::status::StatusRegister;
use crate::rom::Mirroring;

//TODO: provide better getter for registers
pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    oam_addr: u8,
    oam_data: [u8; 256],
    control_register: ControlRegister,
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

impl Interrupt for PPU {
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

    //TODO: ret bool necessary?
    pub fn tick(&mut self, cycles: u8) -> bool  {
        self.cycles += cycles as usize;
        if self.cycles > 341 {
            self.cycles -= 341;
            self.scan_line += 1;

            if self.scan_line == 241 && self.control_register.contains(ControlRegister::GENERATE_NMI) {
                self.status_register.insert(StatusRegister::VERTICAL_BLANK);
                self.outstanding_interrupt = true;
            } else if self.scan_line > 262 {
                self.scan_line = 0;
                self.status_register.remove(StatusRegister::VERTICAL_BLANK);
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
        let before_nmi_status = self.control_register.contains(ControlRegister::GENERATE_NMI);
        self.control_register.update(value);
        if !before_nmi_status && self.control_register.contains(ControlRegister::GENERATE_NMI) && self.status_register.contains(StatusRegister::VERTICAL_BLANK) {
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
        self.status_register.remove(StatusRegister::VERTICAL_BLANK);
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

//TODO: tests