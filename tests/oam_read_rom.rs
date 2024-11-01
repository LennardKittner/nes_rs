use nes_rs::bus::{Bus, Mem};
use nes_rs::cpu::CPU;
use nes_rs::ppu::palette::SystemPalette;
use nes_rs::rom::{Rom};

fn test_rom(path: &str) {
    let rom = Rom::load_from_disk(path).unwrap();

    let bus = Bus::new(rom, SystemPalette::new(), 0f64, |_, _, _| {}, |_, _| {});
    let mut cpu = CPU::new_with_bus(bus);
    cpu.reset();

    let mut phase = 0;
    while phase != 2 {
        cpu.step();
        if cpu.mem_read(0x6000) == 0x80 {
            phase = 1;
        }
        if phase == 1 && cpu.mem_read(0x6000) != 0x80 {
            phase = 2;
        }
    }
    assert_eq!(cpu.mem_read(0x6000), 0)
}

#[test]
fn oam_read() {
    test_rom("./tests/roms/oam_read/oam_read.nes");
}
