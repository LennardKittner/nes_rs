use crate::controller::Controller;
use crate::ppu::pallet::SystemPallet;
use crate::ppu::PPU;
use crate::rendering::frame::Frame;
use crate::rendering::render;
use crate::rom::Rom;

pub struct Bus<'a> {
    cpu_vram: [u8; 2048],
    prg_rom: Vec<u8>,
    ppu: PPU,
    frame: Frame,
    cycles: usize,
    graphics_callback: Box<dyn FnMut(&PPU, &Frame) + 'a>,
    controller_callback: Box<dyn FnMut(&mut Controller, &mut Controller) + 'a>,
    controller_1: Controller,
    controller_2: Controller,
}

impl<'a> Bus<'a> {
    pub fn new<GF, C1F>(rom: Rom, system_palette: SystemPallet, graphics_callback: GF, controller_callback: C1F) -> Bus<'a>
        where GF: FnMut(&PPU, &Frame) + 'a, C1F: FnMut(&mut Controller, &mut Controller) + 'a
    {
        let ppu = PPU::new(rom.chr_rom, rom.screen_mirroring, system_palette);
        Bus {
            cpu_vram: [0; 2048],
            prg_rom: rom.prg_rom,
            cycles: 0,
            ppu,
            frame: Frame::new(),
            graphics_callback: Box::from(graphics_callback),
            controller_callback: Box::from(controller_callback),
            controller_1: Controller::new(),
            controller_2: Controller::new(),
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
        let vblank_before = self.ppu.is_in_vertical_blank();
        let next_scannline = self.ppu.tick(cycles *3);
        let vblank_after = self.ppu.is_in_vertical_blank();

        if !vblank_before && vblank_after {
            render(&self.ppu, &mut self.frame);
            (self.graphics_callback)(&self.ppu, &self.frame);
            (self.controller_callback)(&mut self.controller_1, &mut self.controller_2);
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
            0x4016 => {
                (self.controller_callback)(&mut self.controller_1, &mut self.controller_2);
                self.controller_1.read()
            },
            0x4017 => {
                (self.controller_callback)(&mut self.controller_1, &mut self.controller_2);
                self.controller_2.read()
            },
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => self.read_prg_rom(addr),
            _ => {
                // println!("Ignoring mem read at {addr}");
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
            0x2007 => self.ppu.write_to_data(data),
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_write(mirror_down_addr, data);
            }
            0x4014 => {
                // https://wiki.nesdev.com/w/index.php/PPU_programmer_reference#OAM_DMA_.28.244014.29_.3E_write
                // https://www.nesdev.org/wiki/PPU_OAM#DMA
                // write to oam via dma is directly implemented here instead of using the method from PPU to avoid a buffer and to make it simpler
                if self.cycles % 2 == 0 {
                    self.tick(1);
                } else {
                    self.tick(2);
                }
                let start_address = (data as u16) << 8;
                for i in 0..256 {
                    self.ppu.oam_data[self.ppu.oam_addr as usize] = self.mem_read(start_address + i as u16);
                    self.ppu.oam_addr = self.ppu.oam_addr.wrapping_add(1);
                    self.tick(2);
                }
            }
            0x4016 => {
                self.controller_1.write(data);
                self.controller_2.write(data);
            }
            CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => panic!("Attempt to write to Cartridge ROM space"),
            _ => {
                // println!("Ignoring mem write at 0x{addr:X}");
            }
        }
    }
}

//TODO: tests