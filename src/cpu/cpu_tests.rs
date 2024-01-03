use crate::cpu::{CPU, Flags};

#[test]
fn test_5_ops_working_together() {
    let mut cpu = CPU::new();
    cpu.load_and_run(&vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
    assert_eq!(cpu.register_x, 0xc1);
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
    cpu.load(&vec![0x69, 0x00, 0x00]);
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
    cpu.load(&vec![0x90, 0b0001_0001, 0x00]);
    cpu.reset();
    cpu.set_flag(Flags::Carry);
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8003);
}

#[test]
fn test_bcs_pos() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xB0, 0b0001_0010]);
    cpu.reset();
    cpu.set_flag(Flags::Carry);
    cpu.memory[0x8014] = 0;
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8015);
}

#[test]
fn test_beq_pos() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xF0, 0b0001_0010]);
    cpu.reset();
    cpu.set_flag(Flags::Zero);
    cpu.memory[0x8014] = 0;
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8015);
}

#[test]
fn test_bne_pos() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xD0, 0b0001_0010]);
    cpu.reset();
    cpu.memory[0x8014] = 0;
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8015);
}

#[test]
fn test_bpl_pos() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x10, 0b0001_0010]);
    cpu.reset();
    cpu.memory[0x8014] = 0;
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8015);
}

#[test]
fn test_bmi_pos() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x30, 0b0001_0010]);
    cpu.reset();
    cpu.set_flag(Flags::Negative);
    cpu.memory[0x8014] = 0;
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8015);
}

#[test]
fn test_bvc_pos() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x50, 0b0001_0010]);
    cpu.reset();
    cpu.memory[0x8014] = 0;
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8015);
}

#[test]
fn test_bvs_pos() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x70, 0b0001_0010]);
    cpu.reset();
    cpu.set_flag(Flags::Overflow);
    cpu.memory[0x8014] = 0;
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8015);
}

#[test]
fn test_bit_zero() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x24, 0b0001_0010]);
    cpu.reset();
    cpu.register_a = 0b0011_0000;
    cpu.memory[0x0012] = 0b1000_1100;
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Zero), true);
    assert_eq!(cpu.get_flag(Flags::Overflow), false);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_bit_not_zero() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x24, 0b0001_0010]);
    cpu.reset();
    cpu.register_a = 0b0011_1000;
    cpu.memory[0x0012] = 0b1100_1100;
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Zero), false);
    assert_eq!(cpu.get_flag(Flags::Overflow), true);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_clc() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x18, 0x00]);
    cpu.reset();
    cpu.set_flag(Flags::Carry);
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Carry), false);
}

#[test]
fn test_cld() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xD8, 0x00]);
    cpu.reset();
    cpu.set_flag(Flags::DecimalMode);
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::DecimalMode), false);
}

#[test]
fn test_cli() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x58, 0x00]);
    cpu.reset();
    cpu.set_flag(Flags::InterruptDisabled);
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::InterruptDisabled), false);
}

#[test]
fn test_clv() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xB8, 0x00]);
    cpu.reset();
    cpu.set_flag(Flags::Overflow);
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Overflow), false);
}

#[test]
fn test_cmp_eq() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xC9, 0x50]);
    cpu.reset();
    cpu.register_a = 0x50;
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Carry), true);
    assert_eq!(cpu.get_flag(Flags::Zero), true);
    assert_eq!(cpu.get_flag(Flags::Negative), false);
}

#[test]
fn test_cmp_lt() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xC9, 0x70]);
    cpu.reset();
    cpu.register_a = 0x65;
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Carry), false);
    assert_eq!(cpu.get_flag(Flags::Zero), false);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_cmp_gt() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xC9, 0x30]);
    cpu.reset();
    cpu.register_a = 0x91;
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Carry), true);
    assert_eq!(cpu.get_flag(Flags::Zero), false);
    assert_eq!(cpu.get_flag(Flags::Negative), false);
}

#[test]
fn test_cpx_gt() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xE0, 0x32]);
    cpu.reset();
    cpu.register_x = 0x93;
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Carry), true);
    assert_eq!(cpu.get_flag(Flags::Zero), false);
    assert_eq!(cpu.get_flag(Flags::Negative), false);
}

#[test]
fn test_cpy_lt() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xE0, 0x52]);
    cpu.reset();
    cpu.register_y = 0x33;
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Carry), false);
    assert_eq!(cpu.get_flag(Flags::Zero), false);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_dec_zero() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xD6, 0x11, 0x00]);
    cpu.reset();
    cpu.register_x = 0x23;
    cpu.memory[0x34] = 0x1;
    cpu.run();
    assert_eq!(cpu.memory[0x34], 0x00);
    assert_eq!(cpu.get_flag(Flags::Zero), true);
}

#[test]
fn test_dex_neg() {
    let mut cpu = CPU::new();
    cpu.load_and_run(&vec![0xCA, 0x00]);
    assert_eq!(cpu.register_x, 0xFF);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_dey() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x88, 0x00]);
    cpu.reset();
    cpu.register_y = 0x13;
    cpu.run();
    assert_eq!(cpu.register_y, 0x12);
    assert_eq!(cpu.get_flag(Flags::Negative), false);
    assert_eq!(cpu.get_flag(Flags::Zero), false);
}

#[test]
fn test_inc_neg() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xF6, 0x11, 0x00]);
    cpu.reset();
    cpu.register_x = 0x23;
    cpu.memory[0x34] = 0xEF;
    cpu.run();
    assert_eq!(cpu.memory[0x34], 0xF0);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_inx_overflow() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xE8, 0xE8, 0x00]);
    cpu.reset();
    cpu.register_x = 0xFF;
    cpu.run();
    assert_eq!(cpu.register_x, 1);
}

#[test]
fn test_iny_zero() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xC8, 0x00]);
    cpu.reset();
    cpu.register_y = 0xFF;
    cpu.run();
    assert_eq!(cpu.register_y, 0x00);
    assert_eq!(cpu.get_flag(Flags::Zero), true);
}

#[test]
fn test_jmp_indirect() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x6C, 0x20, 0x01, 0x00]);
    cpu.reset();
    cpu.memory[0x0120] = 0xFC;
    cpu.memory[0x0121] = 0xBA;
    cpu.memory[0xBAFC] = 0x00;
    cpu.run();
    assert_eq!(cpu.program_counter, 0xBAFD);
}

#[test]
fn test_jsr() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0x20, 0x33, 0x45, 0x00]);
    cpu.reset();
    cpu.memory[0x4533] = 0x00;
    cpu.run();
    assert_eq!(cpu.program_counter, 0x4534);
    assert_eq!(cpu.pull_u16(), 0x8002)
}

#[test]
fn test_ldx_zero_page_y() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xB6, 0x3A, 0x00]);
    cpu.reset();
    cpu.register_y = 0x41;
    cpu.memory[0x007B] = 0x76;
    cpu.run();
    assert_eq!(cpu.register_x, 0x76);
}

#[test]
fn test_ldy_zero_absolute_x() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xBC, 0x45, 0x42, 0x00]);
    cpu.reset();
    cpu.register_x = 0x33;
    cpu.memory[0x4278] = 0x17;
    cpu.run();
    assert_eq!(cpu.register_y, 0x17);
}