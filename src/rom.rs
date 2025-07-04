use crate::bus::Mem;
#[cfg(test)]
use crate::cpu::interrupts::RESET_INTERRUPT;
#[cfg(test)]
use crate::mappers::nrom::NROMMapper;
use crate::mappers::{create_mapper, Mapper};
use std::cmp::min;
use std::fs::File;
use std::io::Read;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScree,
    OneScreenLowerBank,
    OneScreenUpperBank,
}

pub enum Region {
    NTSC,
    PAL,
    Multi,
    Dendy,
}

pub struct Rom {
    pub header: RomHeader,
    mapper: Box<dyn Mapper>,
}

const NES_TAG: &str = "NES\x1A";
pub const PRG_ROM_PAGE_SIZE: usize = 16384;
pub const PRG_RAM_PAGE_SIZE: usize = 8192;

pub const CHR_ROM_PAGE_SIZE: usize = 8192;
pub const HEADER_SIZE: usize = 16;

pub enum RomHeader {
    INES(INESHeader),
    NES2(NES2Header),
}

impl RomHeader {
    pub fn get_mapper_number(&self) -> usize {
        match self {
            RomHeader::INES(header) => header.mapper_number,
            RomHeader::NES2(header) => header.mapper_number,
        }
    }
}

pub struct INESHeader {
    pub prg_rom_size: usize,
    pub prg_ram_size: usize,
    pub chr_rom_size: usize,
    pub mirroring: Mirroring,
    pub has_battery_backed_ram: bool,
    pub has_chr_ram: bool,
    /// Unused
    pub has_bus_conflicts: bool,
    /// Unused
    pub has_trainer: bool,
    pub mapper_number: usize,
    pub region: Region,
    pub prg_rom_start: usize,
    pub chr_rom_start: usize,
}

pub struct NES2Header {
    pub prg_rom_size: usize,
    pub prg_rom_start: usize,
    pub prg_ram_size: usize,
    pub prg_nvram_size: usize,
    pub chr_rom_size: usize,
    pub chr_rom_start: usize,
    pub chr_ram_size: usize,
    pub chr_nvram_size: usize,
    pub mirroring: Mirroring,
    pub has_battery_backed_ram: bool,
    pub has_trainer: bool,
    pub mapper_number: usize,
    pub sub_mapper_number: usize,
    pub alternative_nametable: bool,
    pub region: Region,
    /// Unused
    pub system_type: u8,
    /// Unused
    pub num_miscellaneous_roms: usize,
}

impl INESHeader {
    fn new(raw: &[u8]) -> Result<Self, String> {
        let mapper_number = ((raw[7] & 0b1111_0000) | (raw[6] >> 4)) as usize;

        let has_battery_backed_ram = raw[6] & 0b10 != 0;
        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 1 != 0;
        let mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScree,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;
        let has_chr_ram = chr_rom_size == 0;
        let prg_ram_size = min(raw[8] as usize, 1) * PRG_RAM_PAGE_SIZE;

        let has_trainer = raw[6] & 0b100 != 0;

        let prg_rom_start = HEADER_SIZE + if has_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        let has_bus_conflicts = raw[10] & 0b10_0000 != 0;

        let region = if raw[9] & 0b1 == 0 {
            Region::NTSC
        } else {
            Region::PAL
        };

        Ok(INESHeader {
            prg_rom_size,
            chr_rom_size,
            prg_ram_size,
            mirroring,
            has_battery_backed_ram,
            has_chr_ram,
            has_bus_conflicts,
            has_trainer,
            mapper_number,
            region,
            prg_rom_start,
            chr_rom_start,
        })
    }
}

impl NES2Header {
    fn new(raw: &[u8]) -> Result<Self, String> {
        if raw[7] & 0b11 != 0 {
            return Err("Only the NES is supported".to_string());
        }

        let prg_rom_size_lower = raw[4] as usize;
        let prg_rom_size_upper = (raw[9] & 0xF) as usize;
        let prg_rom_size = if prg_rom_size_upper == 0xF {
            let m = prg_rom_size_lower & 0b11;
            let e = prg_rom_size_lower & 0b1111_1100;
            2usize.pow(e as u32) * (m * 2 + 1)
        } else {
            (prg_rom_size_lower | (prg_rom_size_upper << 8)) * PRG_ROM_PAGE_SIZE
        };

        let chr_rom_size_lower = raw[5] as usize;
        let chr_rom_size_upper = (raw[9] & 0xF0) as usize;
        let chr_rom_size = if chr_rom_size_upper == 0xF {
            let m = chr_rom_size_lower & 0b11;
            let e = chr_rom_size_lower & 0b1111_1100;
            2usize.pow(e as u32) * (m * 2 + 1)
        } else {
            (chr_rom_size_lower | (chr_rom_size_upper << 8)) * CHR_ROM_PAGE_SIZE
        };

        let mirroring = if raw[6] & 0b1 == 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };
        let has_battery_backed_ram = raw[6] & 0b10 != 0;
        let has_trainer = raw[6] & 0b100 != 0;
        let alternative_nametable = raw[6] & 0b1000 != 0;

        let mapper_number = (raw[6] & 0xF0) as usize >> 4;
        let mapper_number = mapper_number | (raw[7] & 0xF0) as usize;
        let mapper_number = mapper_number | (((raw[8] & 0xF) as usize) << 8);
        let sub_mapper_number = ((raw[9] & 0xF0) as usize) >> 4;

        let prg_ram_size = 64 << ((raw[10] & 0xF) as usize);
        let prg_nvram_size = 64 << (((raw[10] & 0xF0) as usize) >> 4);

        let chr_ram_size = 64 << ((raw[11] & 0xF) as usize);
        let chr_nvram_size = 64 << (((raw[11] & 0xF0) as usize) >> 4);

        let region = match raw[12] & 0b11 {
            0 => Region::NTSC,
            1 => Region::PAL,
            2 => Region::Multi,
            3 => Region::Dendy,
            _ => unreachable!(),
        };

        let system_type = raw[13];

        let num_miscellaneous_roms = (raw[14] & 0b11) as usize;

        if raw[15] & 0b0011_1111 > 1 {
            return Err("Only the standard NES controller is supported".to_string());
        }

        let prg_rom_start = HEADER_SIZE + if has_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        Ok(NES2Header {
            prg_rom_size,
            prg_rom_start,
            prg_ram_size,
            prg_nvram_size,
            chr_rom_size,
            chr_rom_start,
            chr_ram_size,
            chr_nvram_size,
            mirroring,
            has_battery_backed_ram,
            has_trainer,
            mapper_number,
            sub_mapper_number,
            alternative_nametable,
            region,
            system_type,
            num_miscellaneous_roms,
        })
    }
}

const CARTRIDGE_ROM_AND_MAPPER_START: u16 = 0x8000;
const CARTRIDGE_ROM_AND_MAPPER_END: u16 = 0xFFFF;
const CARTRIDGE_RAM_START: u16 = 0x6000;
const CARTRIDGE_RAM_END: u16 = 0x7FFF;

impl Rom {
    pub fn mem_read(&self, addr: u16) -> Option<u8> {
        match addr {
            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => Some(self.read_cartridge_ram(addr - 0x6000)),
            CARTRIDGE_ROM_AND_MAPPER_START..=CARTRIDGE_ROM_AND_MAPPER_END => {
                Some(self.read_prg_rom(addr - 0x8000))
            }
            _ => None,
        }
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => {
                self.write_cartridge_ram(addr - 0x6000, data)
            }
            CARTRIDGE_ROM_AND_MAPPER_START..=CARTRIDGE_ROM_AND_MAPPER_END => {
                self.mapper_register_write(addr - 0x8000, data)
            }
            _ => (),
        }
    }
}

impl Rom {
    pub fn load_from_disk(path: &str) -> Result<Self, String> {
        let mut file = File::open(path).unwrap();
        let metadata = file.metadata().unwrap();
        let file_size = metadata.len() as usize;
        let mut rom_content = Vec::with_capacity(file_size);
        file.read_to_end(&mut rom_content).unwrap();
        Rom::new(&rom_content)
    }

    #[cfg(test)]
    pub fn new_blank_test_rom(entry_point_address: u16) -> Rom {
        let mut prg_rom = vec![0; PRG_ROM_PAGE_SIZE * 4];
        prg_rom[(RESET_INTERRUPT.interrupt_vector - 0x8000) as usize] = entry_point_address as u8;
        prg_rom[(RESET_INTERRUPT.interrupt_vector - 0x8000 + 1) as usize] =
            (entry_point_address >> 8) as u8;
        let test_header = INESHeader {
            prg_rom_size: prg_rom.len(),
            prg_ram_size: 0,
            chr_rom_size: CHR_ROM_PAGE_SIZE * 4,
            mirroring: Mirroring::Vertical,
            has_battery_backed_ram: false,
            has_chr_ram: true,
            has_bus_conflicts: false,
            has_trainer: false,
            mapper_number: 0,
            region: Region::NTSC,
            prg_rom_start: 0,
            chr_rom_start: prg_rom.len(),
        };
        Rom {
            header: RomHeader::INES(test_header),
            mapper: Box::new(NROMMapper::new(
                prg_rom,
                vec![0; CHR_ROM_PAGE_SIZE * 4],
                true,
                Mirroring::Vertical,
            )),
        }
    }

    pub fn new(raw: &[u8]) -> Result<Self, String> {
        if &raw[0..4] != NES_TAG.as_bytes() {
            return Err("File is not in iNES file format".to_string());
        }
        let ines_ver = (raw[7] >> 2) & 0b11;
        let header = if ines_ver != 0 {
            RomHeader::NES2(NES2Header::new(raw)?)
        } else {
            RomHeader::INES(INESHeader::new(raw)?)
        };
        let mapper = create_mapper(&header, raw);
        Ok(Rom { header, mapper })
    }

    pub fn prg_rom_len(&self) -> usize {
        self.mapper.prg_rom_len()
    }
    pub fn chr_rom_len(&self) -> usize {
        self.mapper.chr_rom_len()
    }
    pub fn read_prg_rom(&self, address: u16) -> u8 {
        self.mapper.read_prg_rom(address)
    }
    pub fn read_chr_rom(&self, address: u16) -> u8 {
        self.mapper.read_chr_rom(address)
    }
    pub fn read_chr_rom_bank(&self, bank: u16, address: u16) -> u8 {
        self.mapper.read_chr_rom_bank(bank, address)
    }
    pub fn read_tile_chr_rom(&self, address: u16) -> &[u8] {
        self.mapper.read_tile_chr_rom(address)
    }
    pub fn read_tile_chr_rom_bank(&self, bank: u16, address: u16) -> &[u8] {
        self.mapper.read_tile_chr_rom_bank(bank, address)
    }
    pub fn mapper_register_write(&mut self, address: u16, value: u8) {
        self.mapper.register_write(address, value);
    }
    pub fn read_cartridge_ram(&self, address: u16) -> u8 {
        self.mapper.read_cartridge_ram(address)
    }
    pub fn write_cartridge_ram(&mut self, address: u16, value: u8) {
        self.mapper.write_cartridge_ram(address, value);
    }
    pub fn get_mirroring_mode(&self) -> Mirroring {
        self.mapper.get_mirroring()
    }
    pub fn write_chr_ram(&mut self, address: u16, value: u8) {
        self.mapper.write_chr_ram(address, value);
    }
}

//TODO: tests
