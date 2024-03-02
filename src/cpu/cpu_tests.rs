use crate::bus::{Bus65k, Mem};
use crate::cpu::{CPU, Flags};

#[test]
fn test_5_ops_working_together() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load_and_run(&[0xa9, 0xc0, 0xaa, 0xe8, 0x00], 0x8000);
    assert_eq!(cpu.register_x, 0xc1);
}

#[test]
fn test_tax() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xaa, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 10;
    cpu.run();
    assert_eq!(cpu.register_x, 10);
}

#[test]
fn test_lda_from_memory() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.mem_write(0x10, 0x55);
    cpu.load_and_run(&[0xa5, 0x10, 0x00], 0x8000);
    assert_eq!(cpu.register_a, 0x55);
}

#[test]
fn test_sta_to_memory() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.mem_write(0x10, 0x55);
    cpu.load_and_run(&[0xA9, 0xFE, 0x85, 0x56, 0x00], 0x8000);
    assert_eq!(cpu.mem_read(0x56), 0xFE);
}

#[test]
fn test_and() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x29, 0b1100_0011, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0b1010_1010;
    cpu.run();
    assert_eq!(cpu.register_a, 0b1000_0010);
}

#[test]
fn test_eor_zero() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x49, 0b1010_1100, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0b1010_1100;
    cpu.run();
    assert_eq!(cpu.register_a, 0);
    assert_eq!(cpu.get_flag(Flags::Zero), true);
}

#[test]
fn test_adc_overflow() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x69, 0b0111_1111, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 1;
    cpu.run();
    assert_eq!(cpu.register_a, 0b1000_0000);
    assert_eq!(cpu.get_flag(Flags::Overflow), true);
}

#[test]
fn test_adc_carry() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x69, 0x00, 0x00], 0x8000);
    cpu.reset();
    cpu.set_flag(Flags::Carry);
    cpu.register_a = 0b1111_1111;
    cpu.run();
    assert_eq!(cpu.register_a, 0);
    assert_eq!(cpu.get_flag(Flags::Carry), true);
}

#[test]
fn test_or_neg() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x49, 0b1011_0000, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0b0000_1100;
    cpu.run();
    assert_eq!(cpu.register_a, 0b1011_1100);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_asl_overflow() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x0a, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0b1000_1100;
    cpu.run();
    assert_eq!(cpu.register_a, 0b0001_1000);
    assert_eq!(cpu.get_flag(Flags::Carry), true);
}

#[test]
fn test_asl_mem_neg() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x0e, 0x30, 0x10], 0x8000);
    cpu.reset();
    cpu.mem_write(0x1030, 0b0100_1001);
    cpu.run();
    assert_eq!(cpu.mem_read(0x1030), 0b1001_0010);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_bcc_neg() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x90, 0b1111_1101], 0x8000);
    cpu.reset();
    cpu.mem_write(0x7999, 0);
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8000);
}

#[test]
fn test_bcc_pos() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x90, 0b0001_0001], 0x8000);
    cpu.reset();
    cpu.mem_write(0x8013, 0);
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8014);
}

#[test]
fn test_bcc_not_taken() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x90, 0b0001_0001, 0x00], 0x8000);
    cpu.reset();
    cpu.set_flag(Flags::Carry);
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8003);
}

#[test]
fn test_bcs_pos() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xB0, 0b0001_0010], 0x8000);
    cpu.reset();
    cpu.set_flag(Flags::Carry);
    cpu.mem_write(0x8014, 0);
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8015);
}

#[test]
fn test_beq_pos() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xF0, 0b0001_0010], 0x8000);
    cpu.reset();
    cpu.set_flag(Flags::Zero);
    cpu.mem_write(0x8014, 0);
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8015);
}

#[test]
fn test_bne_pos() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xD0, 0b0001_0010], 0x8000);
    cpu.reset();
    cpu.mem_write(0x8014, 0);
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8015);
}

#[test]
fn test_bpl_pos() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x10, 0b0001_0010], 0x8000);
    cpu.reset();
    cpu.mem_write(0x8014, 0);
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8015);
}

#[test]
fn test_bmi_pos() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x30, 0b0001_0010], 0x8000);
    cpu.reset();
    cpu.set_flag(Flags::Negative);
    cpu.mem_write(0x8014, 0);
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8015);
}

#[test]
fn test_bvc_pos() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x50, 0b0001_0010], 0x8000);
    cpu.reset();
    cpu.mem_write(0x8014, 0);
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8015);
}

#[test]
fn test_bvs_pos() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x70, 0b0001_0010], 0x8000);
    cpu.reset();
    cpu.set_flag(Flags::Overflow);
    cpu.mem_write(0x8014, 0);
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8015);
}

#[test]
fn test_bit_zero() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x24, 0b0001_0010], 0x8000);
    cpu.reset();
    cpu.register_a = 0b0011_0000;
    cpu.mem_write(0x0012, 0b1000_1100);
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Zero), true);
    assert_eq!(cpu.get_flag(Flags::Overflow), false);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_bit_not_zero() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x24, 0b0001_0010], 0x8000);
    cpu.reset();
    cpu.register_a = 0b0011_1000;
    cpu.mem_write(0x0012, 0b1100_1100);
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Zero), false);
    assert_eq!(cpu.get_flag(Flags::Overflow), true);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_clc() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x18, 0x00], 0x8000);
    cpu.reset();
    cpu.set_flag(Flags::Carry);
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Carry), false);
}

#[test]
fn test_cld() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xD8, 0x00], 0x8000);
    cpu.reset();
    cpu.set_flag(Flags::DecimalMode);
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::DecimalMode), false);
}

#[ignore] // BRK will set InterruptDisabled before the program exits
#[test]
fn test_cli() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x58, 0x00], 0x8000);
    cpu.reset();
    cpu.set_flag(Flags::InterruptDisabled);
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::InterruptDisabled), false);
}

#[test]
fn test_clv() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xB8, 0x00], 0x8000);
    cpu.reset();
    cpu.set_flag(Flags::Overflow);
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Overflow), false);
}

#[test]
fn test_cmp_eq() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xC9, 0x50, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0x50;
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Carry), true);
    assert_eq!(cpu.get_flag(Flags::Zero), true);
    assert_eq!(cpu.get_flag(Flags::Negative), false);
}

#[test]
fn test_cmp_lt() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xC9, 0x70, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0x65;
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Carry), false);
    assert_eq!(cpu.get_flag(Flags::Zero), false);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_cmp_gt() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xC9, 0x30, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0x91;
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Carry), true);
    assert_eq!(cpu.get_flag(Flags::Zero), false);
    assert_eq!(cpu.get_flag(Flags::Negative), false);
}

#[test]
fn test_cpx_gt() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xE0, 0x32, 0x00], 0x8000);
    cpu.reset();
    cpu.register_x = 0x93;
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Carry), true);
    assert_eq!(cpu.get_flag(Flags::Zero), false);
    assert_eq!(cpu.get_flag(Flags::Negative), false);
}

#[test]
fn test_cpy_lt() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xC0, 0x52, 0x00], 0x8000);
    cpu.reset();
    cpu.register_y = 0x33;
    cpu.run();
    assert_eq!(cpu.get_flag(Flags::Carry), false);
    assert_eq!(cpu.get_flag(Flags::Zero), false);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_dec_zero() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xD6, 0x11, 0x00], 0x8000);
    cpu.reset();
    cpu.register_x = 0x23;
    cpu.mem_write(0x34, 0x1);
    cpu.run();
    assert_eq!(cpu.mem_read(0x34), 0x00);
    assert_eq!(cpu.get_flag(Flags::Zero), true);
}

#[test]
fn test_dex_neg() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load_and_run(&[0xCA, 0x00], 0x8000);
    assert_eq!(cpu.register_x, 0xFF);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_dey() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x88, 0x00], 0x8000);
    cpu.reset();
    cpu.register_y = 0x13;
    cpu.run();
    assert_eq!(cpu.register_y, 0x12);
    assert_eq!(cpu.get_flag(Flags::Negative), false);
    assert_eq!(cpu.get_flag(Flags::Zero), false);
}

#[test]
fn test_inc_neg() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xF6, 0x11, 0x00], 0x8000);
    cpu.reset();
    cpu.register_x = 0x23;
    cpu.mem_write(0x34, 0xEF);
    cpu.run();
    assert_eq!(cpu.mem_read(0x34), 0xF0);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_inx_overflow() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xE8, 0xE8, 0x00], 0x8000);
    cpu.reset();
    cpu.register_x = 0xFF;
    cpu.run();
    assert_eq!(cpu.register_x, 1);
}

#[test]
fn test_iny_zero() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xC8, 0x00], 0x8000);
    cpu.reset();
    cpu.register_y = 0xFF;
    cpu.run();
    assert_eq!(cpu.register_y, 0x00);
    assert_eq!(cpu.get_flag(Flags::Zero), true);
}

#[test]
fn test_jmp_indirect() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x6C, 0x20, 0x01, 0x00], 0x8000);
    cpu.reset();
    cpu.mem_write(0x0120, 0xFC);
    cpu.mem_write(0x0121, 0xBA);
    cpu.mem_write(0xBAFC, 0x00);
    cpu.run();
    assert_eq!(cpu.program_counter, 0xBAFD);
}

#[test]
fn test_jsr() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x20, 0x33, 0x45, 0x00], 0x8000);
    cpu.reset();
    cpu.mem_write(0x4533, 0x00);
    cpu.run();
    assert_eq!(cpu.program_counter, 0x4534);
    assert_eq!(cpu.pull_u16(), 0x8002);
    assert_eq!(cpu.register_s, 0xFD);
}

#[test]
fn test_ldx_zero_page_y() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xB6, 0x3A, 0x00], 0x8000);
    cpu.reset();
    cpu.register_y = 0x41;
    cpu.mem_write(0x007B, 0x76);
    cpu.run();
    assert_eq!(cpu.register_x, 0x76);
}

#[test]
fn test_ldy_zero_absolute_x() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xBC, 0x45, 0x42, 0x00], 0x8000);
    cpu.reset();
    cpu.register_x = 0x33;
    cpu.mem_write(0x4278, 0x17);
    cpu.run();
    assert_eq!(cpu.register_y, 0x17);
}

#[test]
fn test_lsr_carry() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x4A, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0b0100_1001;
    cpu.run();
    assert_eq!(cpu.register_a, 0b0010_0100);
    assert_eq!(cpu.get_flag(Flags::Carry), true);
}

#[test]
fn test_lsr_zero_page() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x46, 0xAA,0x00], 0x8000);
    cpu.reset();
    cpu.mem_write(0xAA, 0b0100_1001);
    cpu.run();
    assert_eq!(cpu.mem_read(0xAA), 0b0010_0100);
    assert_eq!(cpu.get_flag(Flags::Carry), true);
}

#[test]
fn test_pha() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x48, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0b0100_1001;
    cpu.run();
    assert_eq!(cpu.pull(), 0b0100_1001);
    assert_eq!(cpu.register_s, 0xFD);

}

#[test]
fn test_php() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x08, 0x00], 0x8000);
    cpu.reset();
    cpu.set_flag(Flags::Carry);
    cpu.set_flag(Flags::Negative);
    cpu.run();
    assert_eq!(cpu.pull(), 0b1011_0101);
    assert_eq!(cpu.register_s, 0xFD);
}

#[test]
fn test_pla() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x68, 0x00], 0x8000);
    cpu.reset();
    cpu.push(0xAB);
    cpu.run();
    assert_eq!(cpu.register_a, 0xAB);
}

#[test]
fn test_plp() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x28, 0x00], 0x8000);
    cpu.reset();
    cpu.push(0b1000_0011);
    cpu.run();
    assert!(cpu.get_flag(Flags::Carry));
    assert!(cpu.get_flag(Flags::Negative));
    assert!(cpu.get_flag(Flags::Zero));
    assert!(!cpu.get_flag(Flags::Overflow));
}

#[test]
fn test_rol_carry_absolute() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x2E, 0x99, 0x55, 0x00], 0x8000);
    cpu.reset();
    cpu.mem_write(0x5599, 0b0100_1000);
    cpu.set_flag(Flags::Carry);
    cpu.run();
    assert_eq!(cpu.mem_read(0x5599), 0b1001_0001);
    assert_eq!(cpu.get_flag(Flags::Carry), false);
}

#[test]
fn test_rol_carry_a() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x2A, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0b0001_1000;
    cpu.set_flag(Flags::Carry);
    cpu.run();
    assert_eq!(cpu.register_a, 0b0011_0001);
    assert_eq!(cpu.get_flag(Flags::Carry), false);
}

#[test]
fn test_ror_carry_neg_absolute() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x6E, 0x99, 0x55, 0x00], 0x8000);
    cpu.reset();
    cpu.mem_write(0x5599, 0b0100_1001);
    cpu.set_flag(Flags::Carry);
    cpu.run();
    assert_eq!(cpu.mem_read(0x5599), 0b1010_0100);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
    assert_eq!(cpu.get_flag(Flags::Carry), true);
}

#[test]
fn test_ror_zero_and_carry_a() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x6A, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0b0000_0001;
    cpu.run();
    assert_eq!(cpu.register_a, 0x00);
    assert_eq!(cpu.get_flag(Flags::Carry), true);
}

#[test]
fn test_rti() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x40, 0x00], 0x8000);
    cpu.reset();
    cpu.push_u16(0xAABB);
    cpu.push(0b1000_0010);
    cpu.mem_write(0xAABB, 0x00);
    cpu.run();
    assert_eq!(cpu.program_counter, 0xAABC);
    assert_eq!(cpu.status, 0b1010_0010);
}

#[test]
fn test_rts() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x60, 0x00], 0x8000);
    cpu.reset();
    cpu.push_u16(0xAABA);
    cpu.mem_write(0xAABB, 0x00);
    cpu.run();
    assert_eq!(cpu.program_counter, 0xAABC);
}

#[test]
fn test_sbc_overflow() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xE9, 0b0000_0010, 0x00], 0x8000);
    cpu.reset();
    cpu.set_flag(Flags::Carry);
    cpu.register_a = 0x01;
    cpu.run();
    assert_eq!(cpu.register_a, 0b1111_1111);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_sbc_sub_zero() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xE9, 0x00, 0x00], 0x8000);
    cpu.reset();
    cpu.set_flag(Flags::Carry);
    cpu.register_a = 0x01;
    cpu.run();
    assert_eq!(cpu.register_a, 0x01);
}

#[test]
fn test_sec() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load_and_run(&[0x38, 0x00], 0x8000);
    assert!(cpu.get_flag(Flags::Carry));
}

#[test]
fn test_sed() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load_and_run(&[0xF8, 0x00], 0x8000);
    assert!(cpu.get_flag(Flags::DecimalMode));
}

#[test]
fn test_sei() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load_and_run(&[0x78, 0x00], 0x8000);
    assert!(cpu.get_flag(Flags::InterruptDisabled));
}

#[test]
fn test_stx_zero_page_y() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x96, 0x24, 0x00], 0x8000);
    cpu.reset();
    cpu.register_y = 0x53;
    cpu.register_x = 0xFE;
    cpu.run();
    assert_eq!(cpu.mem_read(0x77), 0xFE);
}

#[test]
fn test_sty_zero_page_x() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x94, 0x24, 0x00], 0x8000);
    cpu.reset();
    cpu.register_x = 0x53;
    cpu.register_y = 0xFE;
    cpu.run();
    assert_eq!(cpu.mem_read(0x77), 0xFE);
}

#[test]
fn test_tay_neg() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xA8, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0xFF;
    cpu.run();
    assert_eq!(cpu.register_y, 0xFF);
    assert!(cpu.get_flag(Flags::Negative));
}

#[test]
fn test_tsx_neg() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load_and_run(&[0xBA, 0x00], 0x8000);
    assert_eq!(cpu.register_x, 0xFD);
    assert!(cpu.get_flag(Flags::Negative));
}

#[test]
fn test_txa_zero() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x8A, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0x12;
    cpu.run();
    assert_eq!(cpu.register_a, 0x00);
    assert!(cpu.get_flag(Flags::Zero));
}

#[test]
fn test_txs_neg() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x9A, 0x00], 0x8000);
    cpu.reset();
    cpu.register_x = 0xF2;
    cpu.run();
    assert_eq!(cpu.register_s, 0xF2);
    assert!(!cpu.get_flag(Flags::Zero));
}

#[test]
fn test_tya_zero() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x98, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0x12;
    cpu.run();
    assert_eq!(cpu.register_a, 0x00);
    assert!(cpu.get_flag(Flags::Zero));
}

#[test]
fn test_nop() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xEA, 0x00], 0x8000);
    cpu.reset();
    cpu.run();
    assert_eq!(cpu.program_counter, 0x8002);
}

#[test]
fn test_ora_neg_indirect_y() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x11, 0x10, 0x00], 0x8000);
    cpu.reset();
    cpu.mem_write(0x10, 0x22);
    cpu.register_y = 0x20;
    cpu.register_a = 0b0010_1100;
    cpu.mem_write(0x42, 0b1000_0000);
    cpu.run();
    assert_eq!(cpu.register_a, 0b1010_1100);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_ora_neg_indirect_x() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x01, 0x10, 0x00], 0x8000);
    cpu.reset();
    cpu.register_x = 0x20;
    cpu.mem_write(0x30, 0x22);
    cpu.mem_write(0x31, 0x32);
    cpu.register_a = 0b0010_1100;
    cpu.mem_write(0x3222, 0b1000_0000);
    cpu.run();
    assert_eq!(cpu.register_a, 0b1010_1100);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_ora_neg_absolute_y() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x19, 0x10, 0x22, 0x00], 0x8000);
    cpu.reset();
    cpu.register_y = 0x35;
    cpu.mem_write(0x2245, 0b1000_0000);
    cpu.register_a = 0b0010_1100;
    cpu.run();
    assert_eq!(cpu.register_a, 0b1010_1100);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_lax() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xA7, 0x10, 0x00], 0x8000);
    cpu.reset();
    cpu.mem_write(0x10, 0xAB);
    cpu.register_a = 0xFF;
    cpu.register_x = 0xFF;
    cpu.run();
    assert_eq!(cpu.register_a, 0xAB);
    assert_eq!(cpu.register_x, 0xAB);
}

#[test]
fn test_sax() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x87, 0x10, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0b1001_0000;
    cpu.register_x = 0b1000_1000;
    cpu.run();
    assert_eq!(cpu.mem_read(0x10), 0b1000_0000);
    assert!(!cpu.get_flag(Flags::Negative))
}

#[test]
fn test_sbc2_sub_zero() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xEB, 0x00, 0x00], 0x8000);
    cpu.reset();
    cpu.set_flag(Flags::Carry);
    cpu.register_a = 0x01;
    cpu.run();
    assert_eq!(cpu.register_a, 0x01);
}

#[test]
fn test_dcp() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xC7, 0x10, 0x00], 0x8000);
    cpu.reset();
    cpu.mem_write(0x10, 0x05);
    cpu.register_a = 0x02;
    cpu.run();
    assert_eq!(cpu.mem_read(0x10), 0x04);
    assert_eq!(cpu.get_flag(Flags::Carry), false);
    assert_eq!(cpu.get_flag(Flags::Zero), false);
    assert_eq!(cpu.get_flag(Flags::Negative), true);
}

#[test]
fn test_isb() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xE7, 0x10, 0x00], 0x8000);
    cpu.reset();
    cpu.mem_write(0x10, 0x05);
    cpu.register_a = 0x15;
    cpu.run();
    assert_eq!(cpu.mem_read(0x10), 0x06);
    assert_eq!(cpu.register_a, 0x0E)
}

#[test]
fn test_slo() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x07, 0x10, 0x00], 0x8000);
    cpu.reset();
    cpu.mem_write(0x10, 0b0001_0010);
    cpu.register_a = 0b0010_1000;
    cpu.run();
    assert_eq!(cpu.mem_read(0x10), 0b0010_0100);
    assert_eq!(cpu.register_a, 0b0010_1100)
}

#[test]
fn test_rla() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x27, 0x10, 0x00], 0x8000);
    cpu.reset();
    cpu.mem_write(0x10, 0b0001_0010);
    cpu.register_a = 0b0010_1000;
    cpu.set_flag(Flags::Carry);
    cpu.run();
    assert_eq!(cpu.mem_read(0x10), 0b0010_0101);
    assert_eq!(cpu.register_a, 0b0010_0000)
}

#[test]
fn test_sre() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x47, 0x10, 0x00], 0x8000);
    cpu.reset();
    cpu.mem_write(0x10, 0b0001_0010);
    cpu.register_a = 0b0010_1000;
    cpu.run();
    assert_eq!(cpu.mem_read(0x10), 0b0000_1001);
    assert_eq!(cpu.register_a, 0b0010_0001)
}

#[test]
fn test_rra() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x67, 0x10, 0x00], 0x8000);
    cpu.reset();
    cpu.mem_write(0x10, 0b0001_0010);
    cpu.register_a = 0b0010_1000;
    cpu.run();
    assert_eq!(cpu.mem_read(0x10), 0b0000_1001);
    assert_eq!(cpu.register_a, 0b0011_0001)
}

#[test]
fn test_aac() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x0B, 0b1110_1100, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0b1011_1000;
    cpu.run();
    assert!(cpu.get_flag(Flags::Carry));
    assert_eq!(cpu.register_a, 0b1010_1000)
}

#[test]
fn test_asr() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x4B, 0b1110_1100, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0b1011_1000;
    cpu.run();
    assert_eq!(cpu.register_a, 0b0101_0100)
}

#[test]
fn test_arr() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x6B, 0b0000_0001, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0b0100_1001;
    cpu.run();
    assert_eq!(cpu.register_a, 0x00);
    assert_eq!(cpu.get_flag(Flags::Carry), true);
}

#[test]
fn test_xaa() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x8B, 0b1110_1100, 0x00], 0x8000);
    cpu.reset();
    cpu.register_x = 0xFA;
    cpu.register_a = 0b1011_1000;
    cpu.run();
    assert_eq!(cpu.register_x, 0b1011_1000);
    assert_eq!(cpu.register_a, 0b1010_1000);
}

#[test]
fn test_axa() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x9F, 0x00, 0x10, 0x00], 0x8000);
    cpu.reset();
    cpu.register_x = 0b1101_1110;
    cpu.register_y = 0x02;
    cpu.register_a = 0b1001_0010;
    cpu.mem_write(0x1002, 0xFF);
    cpu.run();
    assert_eq!(cpu.mem_read(0x1002), 0b0001_0000);
}

#[test]
fn test_xas() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x9B, 0x10, 0x00, 0x00], 0x8000);
    cpu.reset();
    cpu.register_x = 0b1100_1111;
    cpu.register_y = 0x12;
    cpu.register_a = 0b1000_0011;
    cpu.mem_write(0x22, 0xFF);
    cpu.run();
    assert_eq!(cpu.register_s, 0b1000_0011);
    assert_eq!(cpu.mem_read(0x22), 0b0000_0001);
}

#[test]
fn test_sya() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x9C, 0x00, 0x10, 0x00], 0x8000);
    cpu.reset();
    cpu.register_x = 0x01;
    cpu.register_y = 0b0001_0001;
    cpu.mem_write(0x1001, 0xFF);
    cpu.run();
    assert_eq!(cpu.mem_read(0x1001), 0b0001_0001);
}

#[test]
fn test_sxa() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0x9E, 0x00, 0x10, 0x00], 0x8000);
    cpu.reset();
    cpu.register_y = 0x01;
    cpu.register_x = 0b0001_0001;
    cpu.mem_write(0x1001, 0xFF);
    cpu.run();
    assert_eq!(cpu.mem_read(0x1001), 0b0001_0001);
}

#[test]
fn test_axt() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xAB, 0xCB, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0x01;
    cpu.register_x = 0b0001_0001;
    cpu.run();
    assert_eq!(cpu.register_x, 0xCB);
    assert_eq!(cpu.register_a, 0xCB);
}

#[test]
fn test_lar() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xBB, 0x20, 0x00, 0x00], 0x8000);
    cpu.reset();
    cpu.register_s = 0b1100_1100;
    cpu.register_a = 0xFF;
    cpu.register_x = 0xFF;
    cpu.mem_write(0x20, 0b0111_1011);
    cpu.run();
    assert_eq!(cpu.register_x, 0b0100_1000);
    assert_eq!(cpu.register_a, 0b0100_1000);
    assert_eq!(cpu.register_s, 0b0100_1000);
}

#[test]
fn test_axs() {
    let mut cpu = CPU::new_with_bus(Bus65k::new());
    cpu.load(&[0xCB, 0x02, 0x00], 0x8000);
    cpu.reset();
    cpu.register_a = 0b1100_1110;
    cpu.register_x = 0b1010_1010;
    cpu.run();
    assert_eq!(cpu.register_x, 0b1000_1000);
}