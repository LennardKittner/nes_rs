use itertools::Itertools;
use nes_rs::bus::Bus;
use nes_rs::cpu::CPU;
use nes_rs::ppu::palette::SystemPalette;
use nes_rs::rom::Rom;
use std::env;
use std::str::FromStr;
use std::time::Instant;

fn main() {
    let args = env::args().collect_vec();
    if args.len() < 3 {
        println!(
            "Please provide the path to a NES rom and the number of cycles that should be emulated"
        );
        return;
    }
    let rom_path = &args[1];
    let name = rom_path.split('/').last().unwrap();

    let cycles = i32::from_str(&args[2]).unwrap();

    let bytes: Vec<u8> = std::fs::read(rom_path).unwrap();
    let rom = Rom::new(&bytes).unwrap();

    let palette = SystemPalette::new();

    let bus = Bus::new(rom, palette, 0f64, move |_, _, _| {}, |_, _| {});

    let mut cpu = CPU::new_with_bus(bus);
    cpu.reset();

    let start = Instant::now();
    for _ in 0..cycles {
        cpu.step();
    }
    let duration = start.elapsed();
    println!(
        "Emulated {name} for {cycles}, which took {}s",
        duration.as_secs_f64()
    );
}
