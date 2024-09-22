use crate::mappers::Mapper;

pub struct CNROMMapper {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}

impl CNROMMapper {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Self { prg_rom, chr_rom }
    }
}

impl Mapper for CNROMMapper {
    fn prg_rom_len(&self) -> usize {
        todo!()
    }
    fn chr_rom_len(&self) -> usize {
        todo!()
    }
    fn read_prg_rom(&self, address: u16) -> u8 {
        todo!()
    }
    fn read_chr_rom(&self, address: u16) -> u8 {
        todo!()
    }
    fn read_chr_rom_bank(&self, bank: u16, address: u16) -> u8 {
        todo!()
    }
    fn read_tile_chr_rom(&self, address: u16) -> &[u8] {
        todo!()
    }
    fn read_tile_chr_rom_bank(&self, bank: u16, address: u16) -> &[u8] {
        todo!()
    }
    fn get_current_chr_rom(&self) -> &[u8] {
        todo!()
    }
}