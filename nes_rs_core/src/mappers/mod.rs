use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use crate::mappers::cnrom::{CNROMMapper, CNROMMapperState};
use crate::mappers::mmc1::{MMC1Mapper, MMC1MapperState};
use crate::mappers::nrom::{NROMMapper, NROMMapperState};
use crate::rom::{CHR_ROM_PAGE_SIZE, Mirroring, Rom, RomHeader, RomHeaderWrapper};

pub mod cnrom;
mod mmc1;
pub mod nrom;

#[enum_dispatch]
#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum MapperWrapper {
    CNROMMapper,
    MMC1Mapper,
    NROMMapper,
}

#[enum_dispatch]
#[derive(Serialize, Deserialize, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum MapperStateWrapper {
    CNROMMapperState,
    MMC1MapperState,
    NROMMapperState,
}

#[enum_dispatch(MapperStateWrapper)]
#[allow(dead_code)]
pub trait MapperState: Serialize {}

#[enum_dispatch(MapperWrapper)]
pub trait Mapper {
    fn prg_rom_len(&self) -> usize;
    fn chr_rom_len(&self) -> usize;
    fn read_prg_rom(&self, address: u16) -> u8;
    fn read_chr_rom(&self, address: u16) -> u8;
    fn read_chr_rom_bank(&self, bank: u16, address: u16) -> u8;
    fn read_tile_chr_rom(&self, address: u16) -> &[u8];
    fn read_tile_chr_rom_bank(&self, bank: u16, address: u16) -> &[u8];
    fn register_write(&mut self, _address: u16, _value: u8) {
        println!("register_write not implemented")
    }
    fn read_cartridge_ram(&self, _address: u16) -> u8 {
        0
    }
    fn write_cartridge_ram(&mut self, _address: u16, _value: u8) {
        println!("write_cartridge_ram not implemented")
    }
    fn write_chr_ram(&mut self, _address: u16, _value: u8) {
        println!("write_chr_ram not implemented")
    }
    #[allow(dead_code)]
    fn get_battery_backed_ram(&self) -> Option<&[u8]> {
        None
    }
    fn get_mirroring(&self) -> Mirroring;
    fn get_state(&self) -> MapperStateWrapper;
}

fn get_chr_space(header: &RomHeaderWrapper, raw: &[u8]) -> Vec<u8> {
    match header {
        RomHeaderWrapper::INESHeader(header) => {
            if header.get_has_chr_ram() {
                [0u8; 4 * CHR_ROM_PAGE_SIZE].to_vec()
            } else {
                raw[header.get_chr_rom_start()
                    ..(header.get_chr_rom_start() + header.get_chr_rom_size())]
                    .to_vec()
            }
        }
        RomHeaderWrapper::NES2Header(header) => {
            if header.get_has_chr_ram() {
                vec![0u8; header.get_chr_ram_size().unwrap() * CHR_ROM_PAGE_SIZE]
            } else {
                raw[header.get_chr_rom_start()
                    ..(header.get_chr_rom_start() + header.get_chr_rom_size())]
                    .to_vec()
            }
        }
    }
}

pub fn from_state(mapper_state: MapperStateWrapper, rom: Rom) -> Option<MapperWrapper> {
    Some(match mapper_state {
        MapperStateWrapper::CNROMMapperState(state) => CNROMMapper::from_state(state, rom)?.into(),
        MapperStateWrapper::MMC1MapperState(state) => MMC1Mapper::from_state(state, rom)?.into(),
        MapperStateWrapper::NROMMapperState(state) => NROMMapper::from_state(state, rom)?.into(),
    })
}

pub fn create_mapper(header: &RomHeaderWrapper, raw: &[u8]) -> MapperWrapper {
    let prg_rom_start = header.get_prg_rom_start();
    let prg_rom_size = header.get_prg_rom_size();
    let prg_ram_size = header.get_prg_ram_size();
    let prg_rom = raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec();

    match header.get_mapper_number() {
        0 => NROMMapper::new(
            prg_rom,
            get_chr_space(header, raw),
            prg_ram_size,
            header.get_has_chr_ram(),
            header.get_mirroring(),
        )
        .into(),
        1 => MMC1Mapper::new(
            prg_rom,
            get_chr_space(header, raw),
            header.get_battery_backed_ram_path(),
        )
        .into(),
        3 => CNROMMapper::new(
            prg_rom,
            get_chr_space(header, raw),
            header.get_has_chr_ram(),
            header.get_mirroring(),
        )
        .into(),
        _ => panic!("Mapper not implemented"),
    }
}
