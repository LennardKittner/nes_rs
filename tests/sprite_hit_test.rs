use nes_rs::bus::Bus;
use nes_rs::cpu::CPU;
use nes_rs::ppu::palette::SystemPalette;
use nes_rs::rom::{Mirroring, Rom};
use std::sync::{Arc, Mutex};

fn test_rom(path: &str) {
    let rom = Rom::load_from_disk(path).unwrap();
    let mirroring = rom.get_mirroring_mode();

    let tile1 = Arc::new(Mutex::new(0));
    let error_code_higher = Arc::new(Mutex::new(0));
    let error_code_lower = Arc::new(Mutex::new(0));

    let bus = Bus::new(
        rom,
        SystemPalette::new(),
        0f64,
        |ppu, _, _, _| {
            let (main_name_table, _) = match (mirroring, ppu.address_register.get_name_table()) {
                (Mirroring::Vertical, 0b00)
                | (Mirroring::Vertical, 0b10)
                | (Mirroring::Horizontal, 0b00)
                | (Mirroring::Horizontal, 0b01) => (&ppu.vram[0..0x400], &ppu.vram[0x400..0x800]),
                (Mirroring::Vertical, 0b01)
                | (Mirroring::Vertical, 0b11)
                | (Mirroring::Horizontal, 0b10)
                | (Mirroring::Horizontal, 0b11) => (&ppu.vram[0x400..0x800], &ppu.vram[0..0x400]),
                (_, _) => panic!("Unsupported mirroring mode: {:?}", mirroring),
            };

            *tile1.lock().unwrap() = main_name_table[32 * 6 + 2] as u16;
            *error_code_higher.lock().unwrap() = main_name_table[32 * 6 + 10] as u16;
            *error_code_lower.lock().unwrap() = main_name_table[32 * 6 + 11] as u16;
        },
        |_, _, _| {},
    );

    let mut cpu = CPU::new_with_bus(bus);
    cpu.reset();

    while *tile1.lock().unwrap() <= 0x20 {
        cpu.step();
    }
    if *tile1.lock().unwrap() != 0x50 {
        if *error_code_lower.lock().unwrap() > 0x30 {
            println!(
                "error code {}{}",
                *error_code_higher.lock().unwrap() - 0x30,
                *error_code_lower.lock().unwrap() - 0x30
            );
        } else {
            println!("error code {}", *error_code_higher.lock().unwrap() - 0x30);
        }
    }
    assert_eq!(*tile1.lock().unwrap(), 0x50);
}

#[test]
fn basics() {
    test_rom("tests/roms/sprite_hit_tests_2005.10.05/01.basics.nes");
}

#[ignore]
#[test]
fn alignment() {
    test_rom("tests/roms/sprite_hit_tests_2005.10.05/02.alignment.nes");
}

#[ignore]
#[test]
fn corners() {
    test_rom("tests/roms/sprite_hit_tests_2005.10.05/03.corners.nes");
}

#[ignore]
#[test]
fn flip() {
    test_rom("tests/roms/sprite_hit_tests_2005.10.05/04.flip.nes");
}

#[ignore]
#[test]
fn left_clip() {
    test_rom("tests/roms/sprite_hit_tests_2005.10.05/05.left_clip.nes");
}

#[ignore]
#[test]
fn right_edge() {
    test_rom("tests/roms/sprite_hit_tests_2005.10.05/06.right_edge.nes");
}

#[ignore]
#[test]
fn screen_bottom() {
    test_rom("tests/roms/sprite_hit_tests_2005.10.05/07.screen_bottom.nes");
}

#[ignore] // 8x16 sprites not implemented
#[test]
fn double_height() {
    test_rom("tests/roms/sprite_hit_tests_2005.10.05/08.double_height.nes");
}

#[ignore]
#[test]
fn timing_basics() {
    test_rom("tests/roms/sprite_hit_tests_2005.10.05/09.timing_basics.nes");
}

#[ignore]
#[test]
fn timing_order() {
    test_rom("tests/roms/sprite_hit_tests_2005.10.05/10.timing_order.nes");
}

#[ignore]
#[test]
fn edge_timing() {
    test_rom("tests/roms/sprite_hit_tests_2005.10.05/11.edge_timing.nes");
}
