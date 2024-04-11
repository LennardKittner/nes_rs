use std::fs;
use itertools::Itertools;
use nes_rs::bus::{Bus, Mem};
use nes_rs::cpu::CPU;
use nes_rs::ppu::SYSTEM_PALLET;
use nes_rs::rom::Rom;
use nes_rs::trace::trace;


fn test_rom() -> Rom {
    Rom::load_from_disk("./nestest.nes").unwrap()
}

//TODO: test cycles and PPU and rest of instructions
#[test]
#[should_panic(expected = "APU not implemented")]
fn test_against_nes_test_log_no_ppu() {
    let bus = Bus::new(test_rom(), SYSTEM_PALLET, | _, _ | {}, | _, _ | {});
    let mut cpu = CPU::new_with_bus(bus);
    cpu.reset();
    cpu.program_counter = 0xC000;
    let mut line_num = 0;
    let file_content = fs::read_to_string("./nestest.log").unwrap();
    let test_file = file_content.lines().map(|line| {
        line.split(" PPU").collect_vec()[0]
    }).collect_vec();

    cpu.run_with_callback(|cpu| {
        if cpu.program_counter == 0xC68B {
            panic!("APU not implemented")
        }
        assert_eq!(test_file[line_num], trace(cpu));
        line_num += 1;
        println!("{}", trace(cpu));
    });
}

#[test]
fn test_format_trace() {
    let mut bus = Bus::new(test_rom(), SYSTEM_PALLET, | _, _ | {}, | _, _ | {});
    bus.mem_write(100, 0xA2);
    bus.mem_write(101, 0x01);
    bus.mem_write(102, 0xCA);
    bus.mem_write(103, 0x88);
    bus.mem_write(104, 0x00);

    let mut cpu = CPU::new_with_bus(bus);
    cpu.program_counter = 0x64;
    cpu.register_a = 1;
    cpu.register_x = 2;
    cpu.register_y = 3;

    let mut result = Vec::new();
    cpu.run_with_callback(|cpu| {
        result.push(trace(cpu));
    });
    assert_eq!(
        "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
        result[0]
    );
    assert_eq!(
        "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
        result[1]
    );
    assert_eq!(
        "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
        result[2]
    );
}

#[test]
fn test_mem_access() {
    let mut bus = Bus::new(test_rom(), SYSTEM_PALLET, | _, _ | {}, | _, _ | {});
    // ORA ($33), Y
    bus.mem_write(100, 0x11);
    bus.mem_write(101, 0x33);

    //data
    bus.mem_write(0x33, 0);
    bus.mem_write(0x34, 4);

    //target cell
    bus.mem_write(0x400, 0xAA);

    let mut cpu = CPU::new_with_bus(bus);
    cpu.program_counter = 0x64;
    cpu.register_y = 0;
    let mut result = Vec::new();
    cpu.run_with_callback(|cpu| {
        result.push(trace(cpu));
    });
    assert_eq!(
        "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
        result[0]
    );
}