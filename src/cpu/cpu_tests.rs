use crate::cpu::{CPU, Flags};

#[test]
fn test_5_ops_working_together() {
    let mut cpu = CPU::new();
    cpu.load_and_run(&vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
    assert_eq!(cpu.register_x, 0xc1);
}

#[test]
fn test_inx_overflow() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xe8, 0xe8, 0x00]);
    cpu.reset();
    cpu.register_x = 0xff;
    cpu.run();
    assert_eq!(cpu.register_x, 1);
}

#[test]
fn test_tax() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xaa, 0x00]);
    cpu.reset();
    cpu.register_a = 10;
    cpu.run();
    assert_eq!(cpu.register_x, 10);
}

#[test]
fn test_lda_from_memory() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x10, 0x55);
    cpu.load_and_run(&vec![0xa5, 0x10, 0x00]);
    assert_eq!(cpu.register_a, 0x55);
}

#[test]
fn test_sta_to_memory() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x10, 0x55);
    cpu.load_and_run(&vec![0xA9, 0xFE, 0x85, 0x56, 0x00]);
    assert_eq!(cpu.memory[0x56], 0xFE);
}

#[test]
fn test_and() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x29, 0b1100_0011, 0x00]);
    cpu.reset();
    cpu.register_a = 0b1010_1010;
    cpu.run();
    assert_eq!(cpu.register_a, 0b1000_0010);
}

#[test]
fn test_eor_zero() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x49, 0b1010_1100, 0x00]);
    cpu.reset();
    cpu.register_a = 0b1010_1100;
    cpu.run();
    assert_eq!(cpu.register_a, 0);
    assert_eq!(cpu.get_flag(Flags::Zero), true);
}

#[test]
fn test_adc_overflow() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x69, 0b0111_1111, 0x00]);
    cpu.reset();
    cpu.register_a = 1;
    cpu.run();
    assert_eq!(cpu.register_a, 0b1000_0000);
    assert_eq!(cpu.get_flag(Flags::Overflow), true);
}

#[test]
fn test_adc_carry() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x69, 0, 0x00]);
    cpu.reset();
    cpu.set_flag(Flags::Carry);
    cpu.register_a = 0b1111_1111;
    cpu.run();
    assert_eq!(cpu.register_a, 0);
    assert_eq!(cpu.get_flag(Flags::Carry), true);
}

#[test]
fn test_or_neg() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x49, 0b1011_0000, 0x00]);
    cpu.reset();
    cpu.register_a = 0b0000_1100;
    cpu.run();
    assert_eq!(cpu.register_a, 0b1011_1100);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_asl_overflow() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x0a, 0x00]);
    cpu.reset();
    cpu.register_a = 0b1000_1100;
    cpu.run();
    assert_eq!(cpu.register_a, 0b0001_1000);
    assert_eq!(cpu.get_flag(Flags::Carry), true);
}

#[test]
fn test_asl_mem_neg() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x0e, 0x30, 0x10]);
    cpu.reset();
    cpu.memory[0x1030] = 0b0100_1001;
    cpu.run();
    assert_eq!(cpu.memory[0x1030], 0b1001_0010);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_bcc_neg() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x90, 0b1111_1101]);
    cpu.reset();
    cpu.memory[0x7999] = 0;
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8000);
}

#[test]
fn test_bcc_pos() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x90, 0b0001_0001]);
    cpu.reset();
    cpu.memory[0x8013] = 0;
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8014);
}

#[test]
fn test_bcc_not_taken() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x90, 0b0001_0001, 0]);
    cpu.reset();
    cpu.set_flag(Flags::Carry);
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8003);
}