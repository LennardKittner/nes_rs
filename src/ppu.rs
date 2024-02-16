use crate::rom::Mirroring;

// 7  bit  0
// ---- ----
// VPHB SINN
// |||| ||||
// |||| ||++- Base nametable address
// |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
// |||| |+--- VRAM address increment per CPU read/write of PPUDATA
// |||| |     (0: add 1, going across; 1: add 32, going down)
// |||| +---- Sprite pattern table address for 8x8 sprites
// ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
// |||+------ Background pattern table address (0: $0000; 1: $1000)
// ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
// |+-------- PPU master/slave select
// |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
// +--------- Generate an NMI at the start of the
//            vertical blanking interval (0: off; 1: on)
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum Flags {
    NAMETABLE1              = 0b00000001,
    NAMETABLE2              = 0b00000010,
    VRAM_ADD_INCREMENT      = 0b00000100,
    SPRITE_PATTERN_ADDR     = 0b00001000,
    BACKROUND_PATTERN_ADDR  = 0b00010000,
    SPRITE_SIZE             = 0b00100000,
    MASTER_SLAVE_SELECT     = 0b01000000,
    GENERATE_NMI            = 0b10000000,
}


pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    pub control_register: u8,
    pub mirroring: Mirroring,
    addr: AddressRegister,
    internal_data_buffer: u8,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        PPU {
            chr_rom,
            palette_table: [0; 32],
            vram: [0; 2048],
            oam_data: [0; 256],
            control_register: 0,
            mirroring,
            addr: AddressRegister::new(),
            internal_data_buffer: 0
        }
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(
            if self.get_flag(Flags::VRAM_ADD_INCREMENT) {
                32
            } else {
                1
            }
        );
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
        let addr = self.addr.get();
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
        let addr = self.addr.get();
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

    pub fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    fn get_flag(&self, flag: Flags) -> bool {
        self.control_register & flag as u8 != 0
    }

    fn clear_flag(&mut self, flag: Flags) {
        self.control_register &= !(flag as u8);
    }

    fn set_flag(&mut self, flag: Flags) {
        self.control_register |= flag as u8;
    }

    fn update_flag(&mut self, flag: Flags, set: bool) {
        if set {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }
}

pub struct AddressRegister {
    value: (u8, u8), // 16bit address high byte first, low bytes second
    hi_ptr: bool,
}

impl AddressRegister {
    pub fn new() -> Self {
        AddressRegister {
            value: (0, 0),
            hi_ptr: true,
        }
    }

    fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8;
        self.value.1 = (data & 0xFF) as u8;
    }

    pub fn update(&mut self, data: u8) {
        if self.hi_ptr {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }

        // mirror down address above 0x3FFF
        if self.get() > 0x3FFF {
            self.set(self.get() & 0b11_1111_1111_1111)
        }
        self.hi_ptr = !self.hi_ptr;
    }

    pub fn increment(&mut self, inc: u8) {
        let carry;
        (self.value.1, carry) = self.value.1.overflowing_add(inc);
        if carry {
            self.value.0 = self.value.0.wrapping_add(1);
        }

        // mirror down address above 0x3FFF
        if self.get() > 0x3FFF {
            self.set(self.get() & 0b11_1111_1111_1111)
        }
    }

    pub fn reset_latch(&mut self) {
        self.hi_ptr = true;
    }

    pub fn get(&self) -> u16 {
        (self.value.0 as u16) << 8 | (self.value.1 as u16)
    }
}