use bitflags::bitflags;

bitflags! {
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
    pub struct ControlRegister: u8 {
        const NAMETABLE1              = 0b00000001;
        const NAMETABLE2              = 0b00000010;
        const VRAM_ADD_INCREMENT      = 0b00000100;
        const SPRITE_PATTERN_ADDR     = 0b00001000;
        const BACKROUND_PATTERN_ADDR  = 0b00010000;
        const SPRITE_SIZE             = 0b00100000;
        const MASTER_SLAVE_SELECT     = 0b01000000;
        const GENERATE_NMI            = 0b10000000;
    }
}

impl ControlRegister {
    pub fn new() -> Self {
        Self::empty()
    }

    pub fn update(&mut self, data: u8) {
        self.0 = Self::from_bits(data).unwrap().0;
    }

    pub fn get_nametable_base(&self) -> u16 {
        let nt_1 = self.contains(Self::NAMETABLE1);
        let nt_2 = self.contains(Self::NAMETABLE2);
        match (nt_2, nt_1) {
            (false, false) => 0x2000,
            (false, true)  => 0x2400,
            (true, false)  => 0x2800,
            (true, true)   => 0x2C00,
        }
    }

    pub fn get_vram_increment(&self) -> u8 {
        if self.contains(ControlRegister::VRAM_ADD_INCREMENT) {
            32
        } else {
            1
        }
    }

    pub fn get_sprite_pattern_table_address(&self) -> u16 {
        if self.contains(Self::SPRITE_PATTERN_ADDR) {
            0x1000
        } else {
            0
        }
    }

    pub fn get_background_pattern_table_address(&self) -> u16 {
        if self.contains(Self::BACKROUND_PATTERN_ADDR) {
            0x1000
        } else {
            0
        }
    }

    pub fn get_sprite_size(&self) -> u8 {
        if self.contains(Self::SPRITE_SIZE) {
            16
        } else {
            8
        }
    }
    
    pub fn is_slave(&self) -> bool {
        self.contains(Self::MASTER_SLAVE_SELECT)
    }
    
    pub fn generate_nmi(&self) -> bool {
        self.contains(Self::GENERATE_NMI)
    }
}

impl Default for ControlRegister {
    fn default() -> Self {
        Self::new()
    }
}