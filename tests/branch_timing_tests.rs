use nes_rs::bus::Bus;
use nes_rs::cpu::CPU;
use nes_rs::ppu::palette::SystemPalette;
use nes_rs::rom::{Mirroring, Rom};
use std::sync::{Arc, Mutex};

fn test_rom(path: &str) {
    let rom = Rom::load_from_disk(path).unwrap();
    let mirroring = rom.screen_mirroring;

    let tile1 = Arc::new(Mutex::new(0));

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

            *tile1.lock().unwrap() = main_name_table[32 * 6 + 2] as u16;
        },
        |_, _| {},
    );
    let mut cpu = CPU::new_with_bus(bus);
    cpu.reset();

    while *tile1.lock().unwrap() <= 0x20 {
        cpu.step();
    }
    assert_eq!(*tile1.lock().unwrap(), 0x50);
}

#[test]
fn branch_basics() {
    test_rom("tests/roms/branch_timing_tests/1.Branch_Basics.nes");
}

#[test]
fn backward_branch() {
    test_rom("tests/roms/branch_timing_tests/2.Backward_Branch.nes");
}

#[test]
fn forward_branch() {
    test_rom("tests/roms/branch_timing_tests/3.Forward_Branch.nes");
}
