use crate::mappers::cnrom::CNROMMapper;
use crate::mappers::mmc1::MMC1Mapper;
use crate::mappers::nrom::NROMMapper;
use crate::rom::{Mirroring, RomHeader, CHR_ROM_PAGE_SIZE};

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
    fn get_battery_backed_ram(&self) -> Option<&[u8]> {
        None
    }
    fn get_mirroring(&self) -> Mirroring;
}

fn get_chr_space(header: &RomHeader, raw: &[u8]) -> Vec<u8> {
    match header {
        RomHeader::INES(header) => if header.has_chr_ram {
            &[0u8; 4 * CHR_ROM_PAGE_SIZE]
        } else {
            &raw[header.chr_rom_start..(header.chr_rom_start + header.chr_rom_size)]
        }
        .to_vec(),
        //TODO: nvram and other stuff
        RomHeader::NES2(header) => {
            if header.chr_ram_size > 0 {
                vec![0u8; header.chr_ram_size * CHR_ROM_PAGE_SIZE]
            } else {
                raw[header.chr_rom_start..(header.chr_rom_start + header.chr_rom_size)].to_vec()
            }
        }
    }
}

pub fn create_mapper(rom_header: &RomHeader, raw: &[u8]) -> Box<dyn Mapper> {
    match rom_header.get_mapper_number() {
        0 => match rom_header {
            RomHeader::INES(header) => Box::new(NROMMapper::new(
                raw[header.prg_rom_start..(header.prg_rom_start + header.prg_rom_size)].to_vec(),
                get_chr_space(rom_header, raw),
                header.has_chr_ram,
                header.mirroring,
            )),
            RomHeader::NES2(header) => Box::new(NROMMapper::new(
                raw[header.prg_rom_start..(header.prg_rom_start + header.prg_rom_size)].to_vec(),
                get_chr_space(rom_header, raw),
                header.chr_ram_size > 0,
                header.mirroring,
            )),
        },
        1 => match rom_header {
            RomHeader::INES(header) => Box::new(MMC1Mapper::new(
                raw[header.prg_rom_start..(header.prg_rom_start + header.prg_rom_size)].to_vec(),
                get_chr_space(rom_header, raw),
                header.has_battery_backed_ram,
            )),
            RomHeader::NES2(header) => Box::new(MMC1Mapper::new(
                raw[header.prg_rom_start..(header.prg_rom_start + header.prg_rom_size)].to_vec(),
                get_chr_space(rom_header, raw),
                header.has_battery_backed_ram,
            )),
        },
        3 => match rom_header {
            RomHeader::INES(header) => Box::new(CNROMMapper::new(
                raw[header.prg_rom_start..(header.prg_rom_start + header.prg_rom_size)].to_vec(),
                get_chr_space(rom_header, raw),
                header.has_chr_ram,
                header.mirroring,
            )),
            RomHeader::NES2(header) => Box::new(CNROMMapper::new(
                raw[header.prg_rom_start..(header.prg_rom_start + header.prg_rom_size)].to_vec(),
                get_chr_space(rom_header, raw),
                header.chr_ram_size > 0,
                header.mirroring,
            )),
        },
        _ => panic!("Mapper not implemented"),
    }
}
