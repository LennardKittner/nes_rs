use crate::mappers::cnrom::CNROMMapper;
use crate::mappers::nrom::NROMMapper;

pub mod cnrom;
pub mod nrom;

pub trait Mapper {
    fn prg_rom_len(&self) -> usize;
    fn chr_rom_len(&self) -> usize;
    fn read_prg_rom(&self, address: u16) -> u8;
    fn read_chr_rom(&self, address: u16) -> u8;
    fn read_chr_rom_bank(&self, bank: u16, address: u16) -> u8;
    fn read_tile_chr_rom(&self, address: u16) -> &[u8];
    fn read_tile_chr_rom_bank(&self, bank: u16, address: u16) -> &[u8];
    fn get_current_chr_rom(&self) -> &[u8];
    fn register_write(&mut self, address: u16, value: u8);
}

pub fn create_mapper(idx: u8, prg_rom: &[u8], chr_rom: &[u8]) -> Box<dyn Mapper> {
    match idx {
        0 => Box::new(NROMMapper::new(prg_rom.to_vec(), chr_rom.to_vec())),
        3 => Box::new(CNROMMapper::new(prg_rom.to_vec(), chr_rom.to_vec())),
        _ => panic!("Mapper not implemented"),
    }
}
