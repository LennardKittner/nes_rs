use nes_rs::bus::Bus;
use nes_rs::cpu::CPU;
use nes_rs::ppu::palette::SystemPalette;
use nes_rs::rom::{Mirroring, Rom};
use std::sync::{Arc, Mutex};

fn test_rom(path: &str) {
    let rom = Rom::load_from_disk(path).unwrap();
    let mirroring = rom.screen_mirroring;

    let tile1 = Arc::new(Mutex::new(0));
    let tile2 = Arc::new(Mutex::new(0));

    let bus = Bus::new(
        rom,
        SystemPalette::new(),
        |ppu, _, _| {
            let (main_name_table, _) = match (mirroring, ppu.address_register.get_name_table()) {
                (Mirroring::VERTICAL, 0b00)
                | (Mirroring::VERTICAL, 0b10)
                | (Mirroring::HORIZONTAL, 0b00)
                | (Mirroring::HORIZONTAL, 0b01) => (&ppu.vram[0..0x400], &ppu.vram[0x400..0x800]),
                (Mirroring::VERTICAL, 0b01)
                | (Mirroring::VERTICAL, 0b11)
                | (Mirroring::HORIZONTAL, 0b10)
                | (Mirroring::HORIZONTAL, 0b11) => (&ppu.vram[0x400..0x800], &ppu.vram[0..0x400]),
                (_, _) => panic!("Unsupported mirroring mode: {:?}", mirroring),
            };

            *tile1.lock().unwrap() = main_name_table[32 * 5 + 3] as u16;
            *tile2.lock().unwrap() = main_name_table[32 * 5 + 4] as u16;
        },
        |_, _| {},
    );
    let mut cpu = CPU::new_with_bus(bus);
    cpu.reset();

    while *tile1.lock().unwrap() <= 0x20 || *tile2.lock().unwrap() <= 0x20 {
        //println!("{}", trace(&cpu));
        cpu.step();
    }
    assert_eq!(*tile2.lock().unwrap() - 0x30, 1);
    assert_eq!(*tile1.lock().unwrap() - 0x30, 0);
}

#[test]
fn palette_ram() {
    test_rom("./tests/roms/blargg_ppu_tests_2005.09.15b/palette_ram.nes");
}

#[ignore]
#[test]
/// Ignored because I'm unsure if every NES has the same startup palette
fn power_up_palette() {
    test_rom("./tests/roms/blargg_ppu_tests_2005.09.15b/power_up_palette.nes");
}

#[test]
fn sprite_ram() {
    test_rom("./tests/roms/blargg_ppu_tests_2005.09.15b/sprite_ram.nes");
}

#[test]
fn vram_access() {
    test_rom("./tests/roms/blargg_ppu_tests_2005.09.15b/vram_access.nes");
}

#[test]
fn vbl_clear_time() {
    test_rom("./tests/roms/blargg_ppu_tests_2005.09.15b/vbl_clear_time.nes");
}
