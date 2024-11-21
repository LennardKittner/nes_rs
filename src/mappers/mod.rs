use crate::mappers::cnrom::CNROMMapper;
use crate::mappers::mmc1::MMC1Mapper;
use crate::mappers::nrom::NROMMapper;
use crate::rom::Mirroring;

pub mod cnrom;
mod mmc1;
pub mod nrom;

pub trait Mapper {
    fn prg_rom_len(&self) -> usize;
    fn chr_rom_len(&self) -> usize;
    fn read_prg_rom(&self, address: u16) -> u8;
    fn read_chr_rom(&self, address: u16) -> u8;
    fn read_chr_rom_bank(&self, bank: u16, address: u16) -> u8;
    fn read_tile_chr_rom(&self, address: u16) -> &[u8];
    fn read_tile_chr_rom_bank(&self, bank: u16, address: u16) -> &[u8];
    fn register_write(&mut self, _address: u16, _value: u8) {}
    fn read_cartridge_ram(&self, _address: u16) -> u8 {
        0
    }
    fn write_cartridge_ram(&mut self, _address: u16, _value: u8) {}
    fn write_chr_ram(&mut self, _address: u16, _value: u8) {}
    fn get_battery_backed_ram(&self) -> Option<&[u8]> { None }
    fn get_mirroring(&self) -> Mirroring;
}

pub fn create_mapper(
    idx: u8,
    battery_backed_ram: bool,
    prg_rom: &[u8],
    chr_rom: &[u8],
    has_chr_ram: bool,
    mirroring: Mirroring
) -> Box<dyn Mapper> {
    match idx {
        0 => Box::new(NROMMapper::new(
            prg_rom.to_vec(),
            chr_rom.to_vec(),
            has_chr_ram,
            mirroring,
        )),
        1 => Box::new(MMC1Mapper::new(
            prg_rom.to_vec(),
            chr_rom.to_vec(),
            Some(vec![0u8; 0x2000])
        )),
        3 => Box::new(CNROMMapper::new(
            prg_rom.to_vec(),
            chr_rom.to_vec(),
            has_chr_ram,
            mirroring,
        )),
        _ => panic!("Mapper not implemented"),
    }
}
