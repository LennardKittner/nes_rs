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

enum PRGBankMode {
    Mode32k,
    FirstBankFix,
    LastBankFix,
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

    pub fn get_prg_mode(&self) -> PRGBankMode {
        match (self.value & 0b1100) >> 2 {
            0 | 1 => PRGBankMode::Mode32k,
            2 => PRGBankMode::FirstBankFix,
            3 => PRGBankMode::LastBankFix,
            _ => unreachable!(),
        }
    }

    pub fn chr_8k_mode(&self) -> bool {
        self.value & 0b10000 == 0
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
    prg_rom: Vec<u8>,
    prg_rom_bank0_offset: usize,
    prg_rom_bank1_offset: usize,
    prg_ram: Option<PrgRamWrapper<MmapMut>>,
    chr_rom: Vec<u8>,
    chr_rom_bank0_offset: usize,
    chr_rom_bank1_offset: usize,
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
    const PRG_ADDRESS_MASK: u8 = 0b1111;

    const SHIFT_REGISTER_RESET: u8 = 0b1000_0000;
    const SHIFT_REGISTER_INIT: u8 = 0b1_0000;
    const BANK_SIZE_4K: usize = 0x1000;
    const BANK_SIZE_8K: usize = 0x2000;
    const BANK_SIZE_16K: usize = 0x4000;
    const BANK_SIZE_32K: usize = 0x8000;

    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>, has_battery_backed_ram: bool) -> Self {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open("test.sav")
            .expect("Unable to open save file");

        file.set_len(32768).expect("Unable to resize file");

        let prg_ram = if has_battery_backed_ram {
            Some(PrgRamWrapper::new(unsafe {
                MmapMut::map_mut(&file).expect("Unable to map file")
            }))
        } else {
            None
        };

        let mut mapper = MMC1Mapper {
            prg_rom,
            prg_rom_bank0_offset: 0,
            prg_rom_bank1_offset: 0,
            prg_ram,
            chr_rom,
            chr_rom_bank0_offset: 0,
            chr_rom_bank1_offset: 0,
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

    fn get_prg_bank_offsets(&self) -> (usize, usize) {
        let bank_id = (self.prg_bank_register & Self::PRG_ADDRESS_MASK) as usize;

        let upper_256kb = if self.prg_rom.len() == 524288 {
            ((self.chr_bank0_register | self.chr_bank1_register) & 0b10000) as usize
        } else {
            0usize
        };
        let max_16k_banks = self.prg_rom.len() / Self::BANK_SIZE_16K - 1;
        let max_32k_banks = self.prg_rom.len() / Self::BANK_SIZE_32K - 1;

        let first_bank = upper_256kb * Self::BANK_SIZE_16K;
        let last_bank = if self.prg_rom.len() == 524288 && upper_256kb == 0 {
            (max_16k_banks - 0b10000) * Self::BANK_SIZE_16K
        } else {
            max_16k_banks * Self::BANK_SIZE_16K
        };

        match self.control_register.get_prg_mode() {
            PRGBankMode::Mode32k => (
                Self::BANK_SIZE_32K * (upper_256kb + (bank_id & 0b1110)).min(max_32k_banks),
                Self::BANK_SIZE_32K * (upper_256kb + (bank_id & 0b1110)).min(max_32k_banks)
                    + Self::BANK_SIZE_16K,
            ),
            PRGBankMode::FirstBankFix => (
                first_bank,
                Self::BANK_SIZE_16K * (upper_256kb + bank_id).min(max_16k_banks),
            ),
            PRGBankMode::LastBankFix => (
                Self::BANK_SIZE_16K * (upper_256kb + bank_id).min(max_16k_banks),
                last_bank,
            ),
        }
    }

    fn get_chr_bank_0_offset(&self) -> usize {
        if self.prg_rom.len() == 524288 {
            if self.control_register.chr_8k_mode() {
                0
            } else {
                (self.chr_bank0_register as usize & 1) * Self::BANK_SIZE_4K
            }
        } else {
            //TODO: there are more modes
            if self.control_register.chr_8k_mode() {
                (self.chr_bank0_register as usize >> 1) * Self::BANK_SIZE_8K
            } else {
                self.chr_bank0_register as usize * Self::BANK_SIZE_4K
            }
        }
    }

    fn get_chr_bank_1_offset(&self) -> usize {
        if self.control_register.chr_8k_mode() {
            self.get_chr_bank_0_offset() + Self::BANK_SIZE_4K
        } else {
            self.chr_bank1_register as usize * Self::BANK_SIZE_4K
        }
    }

    fn update_from_registers(&mut self) {
        (self.prg_rom_bank0_offset, self.prg_rom_bank1_offset) = self.get_prg_bank_offsets();
        self.chr_rom_bank0_offset = self.get_chr_bank_0_offset();
        self.chr_rom_bank1_offset = self.get_chr_bank_1_offset();
        self.mirroring = self.control_register.get_mirroring();
    }
}

impl Mapper for MMC1Mapper {
    fn prg_rom_len(&self) -> usize {
        Self::BANK_SIZE_32K
    }

    fn chr_rom_len(&self) -> usize {
        Self::BANK_SIZE_8K
    }

    fn read_prg_rom(&self, address: u16) -> u8 {
        if address < Self::BANK_SIZE_16K as u16 {
            self.prg_rom[address as usize + self.prg_rom_bank0_offset]
        } else {
            self.prg_rom[address as usize - Self::BANK_SIZE_16K + self.prg_rom_bank1_offset]
        }
    }

    fn read_chr_rom(&self, address: u16) -> u8 {
        if address < Self::BANK_SIZE_4K as u16 {
            self.chr_rom[address as usize + self.chr_rom_bank0_offset]
        } else {
            self.chr_rom[address as usize - Self::BANK_SIZE_4K + self.chr_rom_bank1_offset]
        }
    }

    fn read_chr_rom_bank(&self, bank: u16, address: u16) -> u8 {
        self.chr_rom[(bank * Self::BANK_SIZE_8K as u16 + address) as usize]
    }

    fn read_tile_chr_rom(&self, address: u16) -> &[u8] {
        let addr = if address < Self::BANK_SIZE_4K as u16 {
            address as usize + self.chr_rom_bank0_offset
        } else {
            address as usize - Self::BANK_SIZE_4K + self.chr_rom_bank1_offset
        };
        if addr > self.chr_rom.len() {
            return &[0; 16];
        }
        &self.chr_rom[addr..(addr + 16)]
    }

    fn read_tile_chr_rom_bank(&self, bank: u16, address: u16) -> &[u8] {
        let addr = bank as usize * Self::BANK_SIZE_8K + address as usize;
        &self.chr_rom[addr..(addr + 16)]
    }

    ///TODO:: Ignoring consecutive-cycle writes not implemented
    fn register_write(&mut self, address: u16, value: u8) {
        if value & Self::SHIFT_REGISTER_RESET != 0 {
            self.shift_register = Self::SHIFT_REGISTER_INIT;
            self.control_register.reset();
            return;
        }
        let write = self.shift_register & 1 == 1;
        self.shift_register >>= 1;
        self.shift_register |= (value & 1) << 4;
        if write {
            match address + 0x8000 {
                Self::CONTROL_REGISTER_START..=Self::CONTROL_REGISTER_END => {
                    self.control_register.set(self.shift_register);
                }
                Self::CHR_BANK_0_START..=Self::CHR_BANK_0_END => {
                    self.chr_bank0_register = self.shift_register;
                }
                Self::CHR_BANK_1_START..=Self::CHR_BANK_1_END => {
                    self.chr_bank1_register = self.shift_register;
                }
                Self::PRG_BANK_START..=Self::PRG_BANK_END => {
                    self.prg_bank_register = self.shift_register;
                }
                _ => unreachable!(),
            }
            self.update_from_registers();
            self.shift_register = Self::SHIFT_REGISTER_INIT;
        }
    }

    fn read_cartridge_ram(&self, address: u16) -> u8 {
        if let Some(ram) = &self.prg_ram {
            ram[address as usize]
        } else {
            0
        }
    }

    fn write_cartridge_ram(&mut self, address: u16, value: u8) {
        if self.prg_bank_register & 0b10000 != 0 {
            return;
        }
        if let Some(ram) = self.prg_ram.as_mut() {
            ram[address as usize] = value
        } else {
            //println!("Writing to cartridge ram failed.");
        }
    }

    fn write_chr_ram(&mut self, address: u16, value: u8) {
        if address < Self::BANK_SIZE_4K as u16 {
            self.chr_rom[address as usize + self.chr_rom_bank0_offset] = value;
        } else {
            self.chr_rom[address as usize - Self::BANK_SIZE_4K + self.chr_rom_bank1_offset] = value;
        }
    }

    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
