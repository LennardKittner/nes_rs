use crate::rom::Rom;

// A bus addressing 65k of RAM for testing and the snake game
pub struct Bus65k {
    cpu_vram: [u8; 0xFFFF],
}

// Real NES bus
pub struct Bus {
    cpu_vram: [u8; 2048],
    rom: Rom
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        Bus {
            cpu_vram: [0; 2048],
            rom
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        let mut addr = addr - 0x8000;
        if self.rom.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr %= 0x4000;
        }
        self.rom.prg_rom[addr as usize]
    }
}

impl Bus65k {
    pub fn new() -> Self {
        Bus65k {
            cpu_vram: [0; 0xFFFF],
        }
    }
}

pub trait Mem {
    fn mem_read(&self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | lo
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const CARTRIDGE_ROM_START: u16 = 0x8000;
const CARTRIDGE_ROM_END: u16 = 0xFFFF;

impl Mem for Bus {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                todo!("PPU is not implemented yet")
            }
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => self.read_prg_rom(addr),
            _ => {
                println!("Ignoring mem accesses at {addr}");
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                todo!("PPU is not implemented yet")
            }
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => panic!("Attempt to write to Cartridge ROM space"),
            _ => {
                println!("Ignoring mem accesses at {addr}");
            }
        }
    }
}

impl Mem for Bus65k {
    fn mem_read(&self, addr: u16) -> u8 {
        self.cpu_vram[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.cpu_vram[addr as usize] = data;
    }
}

//TODO: tests