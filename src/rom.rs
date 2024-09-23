use std::fs::File;
use std::io::Read;
use crate::mappers::{create_mapper, Mapper};

#[derive(Debug, PartialEq, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum Mirroring {
    VERTICAL,
    HORIZONTAL,
    FOUR_SCREEN,
}

pub struct Rom {
    mapper: Box<dyn Mapper>,
    pub screen_mirroring: Mirroring
}

const NES_TAG: &str = "NES\x1A";
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;
const HEADER_SIZE: usize = 16;

impl Rom {
    pub fn load_from_disk(path: &str) -> Result<Self, String> {
        let mut file = File::open(path).unwrap();
        let metadata = file.metadata().unwrap();
        let file_size = metadata.len() as usize;
        let mut rom_content = Vec::with_capacity(file_size);
        file.read_to_end(&mut rom_content).unwrap();
        Rom::new(&rom_content)
    }

    pub fn new(raw: &[u8]) -> Result<Self, String> {
        if &raw[0..4] != NES_TAG.as_bytes() {
            return Err("File is not in iNES file format".to_string());
        }
        let mapper_idx = (raw[7] & 0b1111_0000) | (raw[6] >> 4);
        let ines_ver = (raw[7] >> 2) & 0b11;
        if ines_ver != 0 {
            return Err("iNES2.0 format is not supported".to_string());
        }

        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 1 != 0;
        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FOUR_SCREEN,
            (false, true) => Mirroring::VERTICAL,
            (false, false) => Mirroring::HORIZONTAL,
        };

        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;

        let skip_trainer = raw[6] & 0b100 != 0;

        let prg_rom_start = HEADER_SIZE + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;
        let mapper = create_mapper(mapper_idx, &raw[prg_rom_start..(prg_rom_start + prg_rom_size)], &raw[chr_rom_start..(chr_rom_start + chr_rom_size)]);
        Ok(Rom {
            mapper,
            screen_mirroring,
        })
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
    pub fn get_current_chr_rom(&self) -> &[u8] {
        self.mapper.get_current_chr_rom()
    }
    pub fn mapper_register_write(&mut self, address: u16, value: u8) {
        self.mapper.register_write(address, value);
    }
}

//TODO: tests