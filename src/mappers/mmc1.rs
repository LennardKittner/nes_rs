use crate::mappers::Mapper;
use crate::rom::Mirroring;
use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::ops;

// 4bit0
// -----
// CPPMM
// |||||
// |||++- Nametable arrangement: (0: one-screen, lower bank; 1: one-screen, upper bank;
// |||               2: horizontal arrangement ("vertical mirroring", PPU A10);
// |||               3: vertical arrangement ("horizontal mirroring", PPU A11) )
// |++--- PRG-ROM bank mode (0, 1: switch 32 KB at $8000, ignoring low bit of bank number;
// |                         2: fix first bank at $8000 and switch 16 KB bank at $C000;
// |                         3: fix last bank at $C000 and switch 16 KB bank at $8000)
// +----- CHR-ROM bank mode (0: switch 8 KB at a time; 1: switch two separate 4 KB banks)

struct ControlRegister {
    pub value: u8,
}

impl ControlRegister {
    pub fn new() -> Self {
        // "Although some tests have found that all versions of the MMC1 seems to reliably power on in the last bank (by setting the "PRG-ROM bank mode" to 3); other tests have found that this is fragile. Several commercial games have reset vectors every 32 KiB, but not every 16, so evidently PRG-ROM bank mode 2 doesn't seem to occur randomly on power-up." nes-dev
        ControlRegister { value: 0b01100 }
    }

    pub fn reset(&mut self) {
        self.value |= 0b01100;
    }

    pub fn set(&mut self, value: u8) {
        self.value = value;
    }

    pub fn get_mirroring(&self) -> Mirroring {
        match self.value & 0b11 {
            0b00 => Mirroring::OneScreenLowerBank,
            0b01 => Mirroring::OneScreenUpperBank,
            0b10 => Mirroring::Vertical,
            0b11 => Mirroring::Horizontal,
            _ => unreachable!(),
        }
    }

    pub fn prg_32k_mode(&self) -> bool {
        self.value & 0b1100 == 0b0100 || self.value & 0b1100 == 0b0000
    }

    pub fn prg_first_bank_fixed(&self) -> bool {
        self.value & 0b1100 == 0b1000
    }

    pub fn prg_last_bank_fixed(&self) -> bool {
        self.value & 0b1100 == 0b1100
    }

    pub fn chr_8k_mode(&self) -> bool {
        self.value & 0b1000 == 0b1000
    }
}

struct PrgRamWrapper<T> {
    data: T,
}

impl<T> PrgRamWrapper<T> {
    pub fn new(data: T) -> Self {
        PrgRamWrapper { data }
    }
}

impl ops::Index<usize> for PrgRamWrapper<Vec<u8>> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        self.data.index(index)
    }
}

impl ops::IndexMut<usize> for PrgRamWrapper<Vec<u8>> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}

impl ops::Index<usize> for PrgRamWrapper<MmapMut> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        self.data.index(index)
    }
}

impl ops::IndexMut<usize> for PrgRamWrapper<MmapMut> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}

pub struct MMC1Mapper {
    has_battery_backed_ram: bool,
    prg_rom: Vec<u8>,
    prg_rom_bank0_offset: usize,
    prg_rom_bank1_offset: usize,
    prg_ram: PrgRamWrapper<MmapMut>,
    chr_rom: Vec<u8>,
    chr_rom_bank0_offset: usize,
    chr_rom_bank1_offset: usize,
    shift_register_pos: u8,
    shift_register: u8,
    control_register: ControlRegister,
    chr_bank0_register: u8,
    chr_bank1_register: u8,
    prg_bank_register: u8,
    mirroring: Mirroring,
}

impl MMC1Mapper {
    const CONTROL_REGISTER_START: u16 = 0x8000;
    const CONTROL_REGISTER_END: u16 = 0x9FFF;
    const CHR_BANK_0_START: u16 = 0xA000;
    const CHR_BANK_0_END: u16 = 0xBFFF;
    const CHR_BANK_1_START: u16 = 0xC000;
    const CHR_BANK_1_END: u16 = 0xDFFF;
    const PRG_BANK_START: u16 = 0xE000;
    const PRG_BANK_END: u16 = 0xFFFF;

    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, prg_ram: Option<Vec<u8>>) -> Self {
        let has_battery_backed_ram;
        let prg_ram = match prg_ram {
            None => {
                has_battery_backed_ram = false;
                vec![]
            }
            Some(v) => {
                has_battery_backed_ram = true;
                v
            }
        };

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("test.sav")
            .expect("Unable to open save file");

        file.set_len(32768).expect("Unable to resize file");

        let mapping = unsafe { MmapMut::map_mut(&file).expect("Unable to map file") };

        let mut mapper = MMC1Mapper {
            has_battery_backed_ram,
            prg_rom,
            prg_rom_bank0_offset: 0,
            prg_rom_bank1_offset: 0,
            prg_ram: PrgRamWrapper::new(mapping),
            chr_rom,
            chr_rom_bank0_offset: 0,
            chr_rom_bank1_offset: 0,
            shift_register_pos: 0,
            shift_register: 0,
            control_register: ControlRegister::new(),
            chr_bank0_register: 0,
            chr_bank1_register: 0,
            prg_bank_register: 0,
            mirroring: Mirroring::Vertical,
        };
        mapper.update_from_registers();
        mapper
    }

    fn get_prg_bank_0_offset(&self) -> usize {
        let mut bank_id = self.prg_bank_register & 0b1111;

        if self.control_register.chr_8k_mode() {
            bank_id += self.chr_bank0_register & 0x10;
        } else {
            bank_id += self.chr_bank1_register & 0x10;
        }

        let mut offset = if self.control_register.prg_first_bank_fixed() {
            0usize
        } else if self.control_register.prg_last_bank_fixed() {
            bank_id as usize * 16384
        } else {
            (bank_id >> 1) as usize * 32768
        };

        if !self.has_battery_backed_ram
            && self.control_register.prg_first_bank_fixed()
            && self.prg_bank_register & 0b10000 != 0
        {
            offset += 16384;
        }

        offset
    }

    fn get_prg_bank_1_offset(&self) -> usize {
        let mut offset = if self.control_register.prg_first_bank_fixed() {
            (self.prg_bank_register & 0b1111) as usize * 16384
        } else if self.control_register.prg_last_bank_fixed() {
            self.prg_rom.len() - 16384
        } else {
            ((self.prg_bank_register & 0b1111) >> 1) as usize * 32768 + 16384
        };

        if !self.has_battery_backed_ram
            && self.control_register.prg_last_bank_fixed()
            && self.prg_bank_register & 0b1000 != 0
        {
            offset -= 16384;
        }
        offset
    }

    fn get_chr_bank_0_offset(&self) -> usize {
        if self.control_register.chr_8k_mode() {
            (self.chr_bank0_register as usize >> 1) * 8192
        } else {
            self.chr_bank0_register as usize * 4096
        }
    }

    fn get_chr_bank_1_offset(&self) -> usize {
        if self.control_register.chr_8k_mode() {
            (self.chr_bank0_register as usize >> 1) * 8192 + 4096
        } else {
            self.chr_bank1_register as usize * 4096
        }
    }

    fn update_from_registers(&mut self) {
        self.prg_rom_bank0_offset = self.get_prg_bank_0_offset();
        self.prg_rom_bank1_offset = self.get_prg_bank_1_offset();
        self.chr_rom_bank0_offset = self.get_chr_bank_0_offset();
        self.chr_rom_bank1_offset = self.get_chr_bank_1_offset();
        self.mirroring = self.control_register.get_mirroring();
    }
}

impl Mapper for MMC1Mapper {
    fn prg_rom_len(&self) -> usize {
        32768
    }

    fn chr_rom_len(&self) -> usize {
        8192
    }

    fn read_prg_rom(&self, address: u16) -> u8 {
        if address < 16384 {
            self.prg_rom[address as usize + self.prg_rom_bank0_offset]
        } else {
            self.prg_rom[(address - 16384) as usize + self.prg_rom_bank1_offset]
        }
    }

    fn read_chr_rom(&self, address: u16) -> u8 {
        if address < 4096 {
            self.chr_rom[address as usize + self.chr_rom_bank0_offset]
        } else {
            self.chr_rom[(address - 4096) as usize + self.chr_rom_bank1_offset]
        }
    }

    fn read_chr_rom_bank(&self, bank: u16, address: u16) -> u8 {
        self.chr_rom[(bank * 8192 + address) as usize]
    }

    fn read_tile_chr_rom(&self, address: u16) -> &[u8] {
        let addr = if address < 4096 {
            address as usize + self.chr_rom_bank0_offset
        } else {
            (address - 4096) as usize + self.chr_rom_bank1_offset
        };
        if addr > self.chr_rom.len() {
            return &[0; 16];
        }
        &self.chr_rom[addr..(addr + 16)]
    }

    fn read_tile_chr_rom_bank(&self, bank: u16, address: u16) -> &[u8] {
        let addr = (bank * 8192 + address) as usize;
        &self.chr_rom[addr..(addr + 16)]
    }

    /// Ignoring consecutive-cycle writes not implemented
    fn register_write(&mut self, address: u16, value: u8) {
        // if value != 192 && value != 129 {
        // println!("{:x}", value);
        // }        // if value != 192 && value != 129 {

        // if address + 0x8000 != 0xffff && address + 0x8000 != 0xbfdf {
        // println!("{:x}", address + 0x8000);
        // }
        if value & 0b1000_0000 != 0 {
            // println!(
            //     "RESET  {:b} {:x} {:b}",
            //     value,
            //     address + 0x8000,
            //     self.shift_register
            // );
            self.shift_register_pos = 0;
            self.shift_register = 0;
            self.control_register.reset();
            return;
        }
        // println!(
        //     "WRITE  {value:b} {value:x} {:x} {:b}",
        //     address + 0x8000,
        //     self.shift_register
        // );
        self.shift_register |= (value & 1) << self.shift_register_pos;
        self.shift_register_pos += 1;
        if self.shift_register_pos >= 5 {
            // println!("WRITE");
            match address + 0x8000 {
                Self::CONTROL_REGISTER_START..=Self::CONTROL_REGISTER_END => {
                    self.control_register.set(self.shift_register);
                }
                Self::CHR_BANK_0_START..=Self::CHR_BANK_0_END => {
                    // if self.shift_register != 0 {
                    //     //println!("wrote chr bank 0 {}", self.shift_register);
                    // }
                    self.chr_bank0_register = self.shift_register;
                }
                Self::CHR_BANK_1_START..=Self::CHR_BANK_1_END => {
                    // if self.shift_register != 0 {
                    //     //println!("wrote chr bank 1 {}", self.shift_register);
                    // }
                    self.chr_bank1_register = self.shift_register;
                }
                Self::PRG_BANK_START..=Self::PRG_BANK_END => {
                    self.prg_bank_register = self.shift_register;
                }
                _ => (),
            }
            self.update_from_registers();
            self.shift_register = 0;
            self.shift_register_pos = 0;
        }
    }

    fn read_cartridge_ram(&self, address: u16) -> u8 {
        if self.has_battery_backed_ram && self.prg_bank_register & 0b10000 != 0 {
            self.prg_ram[address as usize]
        } else {
            0
        }
    }

    fn write_cartridge_ram(&mut self, address: u16, value: u8) {
        if self.has_battery_backed_ram && self.prg_bank_register & 0b10000 != 0 {
            self.prg_ram[address as usize] = value
        } else {
            //println!("Writing to cartridge ram failed.");
        }
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
