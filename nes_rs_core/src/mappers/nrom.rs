use serde::{Deserialize, Serialize};

use crate::mappers::Mapper;
use crate::mappers::MapperStateWrapper;
use crate::mappers::MapperWrapper;
use crate::rom::Mirroring;
use crate::rom::Rom;

const RAM_SIZE: usize = 0x2000;

#[derive(Debug)]
pub struct NROMMapper {
    prg_rom: Vec<u8>,
    chr_space: Vec<u8>,
    has_chr_ram: bool,
    cartridge_ram: Vec<u8>,
    mirroring: Mirroring,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NROMMapperState {
    cartridge_ram: Vec<u8>,
}

impl NROMMapper {
    pub fn new(
        prg_rom: Vec<u8>,
        chr_rom: Vec<u8>,
        prg_ram_size: usize,
        has_chr_ram: bool,
        mirroring: Mirroring,
    ) -> Self {
        let prg_ram_size = if prg_ram_size == 0 {
            RAM_SIZE
        } else {
            prg_ram_size
        };
        Self {
            prg_rom,
            chr_space: chr_rom,
            has_chr_ram,
            cartridge_ram: vec![0; prg_ram_size],
            mirroring,
        }
    }

    pub fn from_state(state: NROMMapperState, rom: Rom) -> Option<NROMMapper> {
        if let MapperWrapper::NROMMapper(mut this) = rom.mapper {
            this.cartridge_ram = state.cartridge_ram;
            Some(this)
        } else {
            None
        }
    }
}

impl Mapper for NROMMapper {
    fn prg_rom_len(&self) -> usize {
        self.prg_rom.len()
    }
    fn chr_rom_len(&self) -> usize {
        self.chr_space.len()
    }
    fn read_prg_rom(&self, address: u16) -> u8 {
        self.prg_rom[address as usize % self.prg_rom_len()]
    }
    fn read_chr_rom(&self, address: u16) -> u8 {
        self.chr_space[address as usize]
    }
    fn read_chr_rom_bank(&self, bank: u16, address: u16) -> u8 {
        self.chr_space[bank as usize * 0x1000 + address as usize]
    }
    fn read_tile_chr_rom(&self, address: u16) -> &[u8] {
        let address = address as usize;
        &self.chr_space[address..(address + 16)]
    }
    fn read_tile_chr_rom_bank(&self, bank: u16, address: u16) -> &[u8] {
        let address = bank as usize * 0x1000 + address as usize;
        &self.chr_space[address..(address + 16)]
    }
    fn read_cartridge_ram(&self, address: u16) -> u8 {
        self.cartridge_ram[address as usize]
    }
    fn write_cartridge_ram(&mut self, address: u16, value: u8) {
        self.cartridge_ram[address as usize] = value;
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
        NROMMapperState {
            cartridge_ram: self.cartridge_ram.clone(),
        }
        .into()
    }
}
