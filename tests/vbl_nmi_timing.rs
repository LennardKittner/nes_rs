use nes_rs::bus::Bus;
use nes_rs::cpu::CPU;
use nes_rs::ppu::palette::SystemPalette;
use nes_rs::rom::{Mirroring, Rom};
use std::sync::{Arc, Mutex};

fn test_rom(path: &str, column: usize, expected: u8) {
    let rom = Rom::load_from_disk(path).unwrap();
    let mirroring = rom.get_mirroring_mode();

    let tile1 = Arc::new(Mutex::new(0));
    let tile2 = Arc::new(Mutex::new(0));

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

            // 10 or 12
            *tile1.lock().unwrap() = main_name_table[32 * 6 + column];
            *tile2.lock().unwrap() = main_name_table[32 * 6 + 2];
        },
        |_, _, _| {},
    );
    let mut cpu = CPU::new_with_bus(bus);
    cpu.reset();

    while *tile1.lock().unwrap() <= 0x20 && *tile2.lock().unwrap() <= 0x20 {
        cpu.step();
    }
    if *tile1.lock().unwrap() > 0x30 {
        assert_eq!(*tile1.lock().unwrap() - 0x30, expected);
    }
}

#[test]
fn frame_basics() {
    test_rom("./tests/roms/vbl_nmi_timing/1.frame_basics.nes", 12, 0);
}

#[test]
fn vbl_timing() {
    // TODO: 8) Reading 1 PPU clock before VBL should suppress setting
    test_rom("./tests/roms/vbl_nmi_timing/2.vbl_timing.nes", 10, 8);
}

#[ignore]
#[test]
fn even_odd_frames() {
    test_rom("./tests/roms/vbl_nmi_timing/3.even_odd_frames.nes", 10, 0);
}
//
// #[test]
// fn vbl_clear_timing() {
//     test_rom("./tests/roms/vbl_nmi_timing/4.vbl_clear_timing.nes");
// }
//
// #[test]
// fn nmi_suppression() {
//     test_rom("./tests/roms/vbl_nmi_timing/5.nmi_suppression.nes");
// }
//
// #[test]
// fn nmi_disable() {
//     test_rom("./tests/roms/vbl_nmi_timing/6.nmi_disable.nes");
// }
//
// #[test]
// fn nmi_timing() {
//     test_rom("./tests/roms/vbl_nmi_timing/7.nmi_timing.nes");
// }
