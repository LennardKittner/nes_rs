use serde::{Deserialize, Serialize};

use crate::mappers::{Mapper, MapperStateWrapper, MapperWrapper};
use crate::rom::{Mirroring, Rom};

#[derive(Debug)]
pub struct CNROMMapper {
    prg_rom: Vec<u8>,
    chr_space: Vec<u8>,
    has_chr_ram: bool,
    current_bank_offset: usize,
    mirroring: Mirroring,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CNROMMapperState {
    current_bank_offset: usize,
}

impl CNROMMapper {
    pub fn new(
        prg_rom: Vec<u8>,
        chr_rom: Vec<u8>,
        has_chr_ram: bool,
        mirroring: Mirroring,
    ) -> Self {
        Self {
            prg_rom,
            chr_space: chr_rom,
            has_chr_ram,
            current_bank_offset: 0,
            mirroring,
        }
    }

    pub fn from_state(state: CNROMMapperState, rom: Rom) -> Option<CNROMMapper> {
        if let MapperWrapper::CNROMMapper(mut this) = rom.mapper {
            this.current_bank_offset = state.current_bank_offset;
            Some(this)
        } else {
            None
        }
    }
}

impl Mapper for CNROMMapper {
    fn prg_rom_len(&self) -> usize {
        self.prg_rom.len()
    }
    fn chr_rom_len(&self) -> usize {
        self.chr_space.len()
    }
    fn read_prg_rom(&self, address: u16) -> u8 {
        self.prg_rom[address as usize]
    }
    fn read_chr_rom(&self, address: u16) -> u8 {
        self.chr_space[self.current_bank_offset + address as usize]
    }
    fn read_chr_rom_bank(&self, bank: u16, address: u16) -> u8 {
        self.chr_space[bank as usize * 0x1000 + address as usize]
    }
    fn read_tile_chr_rom(&self, address: u16) -> &[u8] {
        let address = self.current_bank_offset + address as usize;
        &self.chr_space[address..(address + 16)]
    }
    fn read_tile_chr_rom_bank(&self, bank: u16, address: u16) -> &[u8] {
        let address = bank as usize * 0x1000 + address as usize;
        &self.chr_space[address..(address + 16)]
    }
    fn register_write(&mut self, _address: u16, value: u8) {
        self.current_bank_offset = (value & 0x3) as usize * 0x2000;
    }
    fn write_chr_ram(&mut self, address: u16, value: u8) {
        if self.has_chr_ram {
            self.chr_space[address as usize] = value;
        }
    }
    fn get_mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn get_state(&self) -> MapperStateWrapper {
        CNROMMapperState {
            current_bank_offset: self.current_bank_offset,
        }
        .into()
    }
}
