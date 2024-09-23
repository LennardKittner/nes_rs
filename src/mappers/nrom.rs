use crate::mappers::Mapper;

pub struct NROMMapper {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
}

impl NROMMapper {
    pub fn new(prg_rom: Vec<u8>, chr_rom: Vec<u8>) -> Self {
        Self { prg_rom, chr_rom }
    }
}

impl Mapper for NROMMapper {
    fn prg_rom_len(&self) -> usize {
        self.prg_rom.len()
    }
    fn chr_rom_len(&self) -> usize {
        self.chr_rom.len()
    }
    fn read_prg_rom(&self, address: u16) -> u8 {
        self.prg_rom[address as usize]
    }
    fn read_chr_rom(&self, address: u16) -> u8 {
        self.chr_rom[address as usize]
    }
    fn read_chr_rom_bank(&self, bank: u16, address: u16) -> u8 {
        self.chr_rom[bank as usize * 0x1000 + address as usize]
    }
    fn read_tile_chr_rom(&self, address: u16) -> &[u8] {
        let address = address as usize;
        &self.chr_rom[address..(address + 16)]
    }
    fn read_tile_chr_rom_bank(&self, bank: u16, address: u16) -> &[u8] {
        let address = bank as usize * 0x1000 + address as usize;
        &self.chr_rom[address..(address + 16)]
    }
    fn get_current_chr_rom(&self) -> &[u8] {
        &self.chr_rom
    }
    fn register_write(&mut self, _address: u16, _value: u8) {}
}