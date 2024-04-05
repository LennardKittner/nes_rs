use crate::ppu::PPU;
use crate::rom::Rom;

pub struct Bus<'a> {
    cpu_vram: [u8; 2048],
    prg_rom: Vec<u8>,
    ppu: PPU,
    cycles: usize,
    game_loop_callback: Box<dyn FnMut(&PPU) + 'a>
}

impl<'a> Bus<'a> {
    pub fn new<'b, F>(rom: Rom, game_loop_callback: F) -> Bus<'b>
        where F: FnMut(&PPU) + 'b
    {
        let ppu = PPU::new(rom.chr_rom, rom.screen_mirroring);
        Bus {
            cpu_vram: [0; 2048],
            prg_rom: rom.prg_rom,
            cycles: 0,
            ppu,
            game_loop_callback: Box::from(game_loop_callback)
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        let mut addr = addr - 0x8000;
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr %= 0x4000;
        }
        self.prg_rom[addr as usize]
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;
        let refresh = self.ppu.tick(cycles * 3);
        if refresh {
            (self.game_loop_callback)(&mut self.ppu);
        }
    }
}

pub trait PollInterrupt {
    fn poll_nmi_status(&mut self) -> bool;
}

pub trait Mem {
    fn mem_read(&mut self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
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
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const CARTRIDGE_ROM_START: u16 = 0x8000;
const CARTRIDGE_ROM_END: u16 = 0xFFFF;

impl PollInterrupt for Bus<'_> {
    fn poll_nmi_status(&mut self) -> bool {
        self.ppu.poll_nmi_status()
    }
}

impl Mem for Bus<'_> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                //TODO: maybe detect tracing
                //panic!("Attempt to read from write-only PPU address {:x}", addr);
                0
            }
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read_data(),
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_read(mirror_down_addr)
            }
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => self.read_prg_rom(addr),
            _ => {
                println!("Ignoring mem read at {addr}");
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
            0x2000 => {
                //println!("wrote {:X} to control register", {data});
                self.ppu.write_to_ctrl(data)
            },
            0x2001 => self.ppu.write_to_mask(data),
            0x2002 => panic!("write to PPU status register"),
            0x2003 => self.ppu.write_to_addr(data),
            0x2004 => self.ppu.write_to_data(data),
            0x2005 => self.ppu.write_to_scroll(data),
            0x2006 => self.ppu.write_to_addr(data),
            0x2007 => {
                //println!("write 2007 {data}");
                self.ppu.write_to_data(data)
            },
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_write(mirror_down_addr, data);
            }
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => panic!("Attempt to write to Cartridge ROM space"),
            _ => {
                println!("Ignoring mem write at 0x{addr:X}");
            }
        }
    }
}

//TODO: tests