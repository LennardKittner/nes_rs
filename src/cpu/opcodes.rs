use crate::cpu::{AddressingMode, CPU};
use lazy_static::lazy_static;

pub const BRANCH_OP_CODES: [u8; 8] = [
    0x90u8, 0xB0u8, 0xF0u8, 0x30u8, 0xD0u8, 0x10u8, 0x50u8, 0x70u8,
];

lazy_static! {
    pub static ref CPU_INSTRUCTIONS: [OpCode; 256] = #[allow(clippy::redundant_closure)] { // clippy suggests to use the function reference directly, but this causes an error.
        let mut arr: [OpCode; 256] = [OpCode::new(0, "", 0, 0, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::brk(cpu))); 256];
        // size of brk is 0 so the target jump location is one less
        // 4 cycles because brk already ticks 3 cycles internally
        arr[0x00] = OpCode::new(0x00, "BRK", 0, 4, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::brk(cpu)));
        arr[0xEA] = OpCode::new(0xEA, "NOP", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));

        arr[0x04] = OpCode::new(0x04, "*NOP", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x14] = OpCode::new(0x14, "*NOP", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x34] = OpCode::new(0x34, "*NOP", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x44] = OpCode::new(0x44, "*NOP", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x54] = OpCode::new(0x54, "*NOP", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x64] = OpCode::new(0x64, "*NOP", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x74] = OpCode::new(0x74, "*NOP", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x80] = OpCode::new(0x80, "*NOP", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x82] = OpCode::new(0x82, "*NOP", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x89] = OpCode::new(0x89, "*NOP", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0xC2] = OpCode::new(0xC2, "*NOP", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0xD4] = OpCode::new(0xD4, "*NOP", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0xE2] = OpCode::new(0xE2, "*NOP", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0xF4] = OpCode::new(0xF4, "*NOP", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));

        arr[0x0C] = OpCode::new(0x0C, "*NOP", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x1C] = OpCode::new(0x1C, "*NOP", 3, 4, true, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x3C] = OpCode::new(0x3C, "*NOP", 3, 4, true, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x5C] = OpCode::new(0x5C, "*NOP", 3, 4, true, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x7C] = OpCode::new(0x7C, "*NOP", 3, 4, true, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0xDC] = OpCode::new(0xDC, "*NOP", 3, 4, true, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0xFC] = OpCode::new(0xFC, "*NOP", 3, 4, true, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));

        arr[0x1A] = OpCode::new(0x1A, "*NOP", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x3A] = OpCode::new(0x3A, "*NOP", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x5A] = OpCode::new(0x5A, "*NOP", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0x7A] = OpCode::new(0x7A, "*NOP", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0xDA] = OpCode::new(0xDA, "*NOP", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));
        arr[0xFA] = OpCode::new(0xFA, "*NOP", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpuAndAddressing(|cpu, mode| CPU::nop(cpu, mode)));

        arr[0x02] = OpCode::new(0x02, "*KIL", 1, 0, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::kil(cpu)));
        arr[0x12] = OpCode::new(0x12, "*KIL", 1, 0, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::kil(cpu)));
        arr[0x22] = OpCode::new(0x22, "*KIL", 1, 0, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::kil(cpu)));
        arr[0x32] = OpCode::new(0x32, "*KIL", 1, 0, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::kil(cpu)));
        arr[0x42] = OpCode::new(0x42, "*KIL", 1, 0, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::kil(cpu)));
        arr[0x52] = OpCode::new(0x52, "*KIL", 1, 0, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::kil(cpu)));
        arr[0x62] = OpCode::new(0x62, "*KIL", 1, 0, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::kil(cpu)));
        arr[0x72] = OpCode::new(0x72, "*KIL", 1, 0, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::kil(cpu)));
        arr[0x92] = OpCode::new(0x92, "*KIL", 1, 0, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::kil(cpu)));
        arr[0xB2] = OpCode::new(0xB2, "*KIL", 1, 0, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::kil(cpu)));
        arr[0xD2] = OpCode::new(0xD2, "*KIL", 1, 0, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::kil(cpu)));
        arr[0xF2] = OpCode::new(0xF2, "*KIL", 1, 0, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::kil(cpu)));

        /* Transfers */
        arr[0xAA] = OpCode::new(0xAA, "TAX", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::tax(cpu)));
        arr[0xA8] = OpCode::new(0xA8, "TAY", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::tay(cpu)));
        arr[0xBA] = OpCode::new(0xBA, "TSX", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::tsx(cpu)));
        arr[0x8A] = OpCode::new(0x8A, "TXA", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::txa(cpu)));
        arr[0x9A] = OpCode::new(0x9A, "TXS", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::txs(cpu)));
        arr[0x98] = OpCode::new(0x98, "TYA", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::tya(cpu)));

        /* Loads */
        arr[0xA9] = OpCode::new(0xA9, "LDA", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lda(cpu, mode)));
        arr[0xA5] = OpCode::new(0xA5, "LDA", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lda(cpu, mode)));
        arr[0xB5] = OpCode::new(0xB5, "LDA", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lda(cpu, mode)));
        arr[0xAD] = OpCode::new(0xAD, "LDA", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lda(cpu, mode)));
        arr[0xBD] = OpCode::new(0xBD, "LDA", 3, 4, true, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lda(cpu, mode)));
        arr[0xB9] = OpCode::new(0xB9, "LDA", 3, 4, true, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lda(cpu, mode)));
        arr[0xA1] = OpCode::new(0xA1, "LDA", 2, 6, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lda(cpu, mode)));
        arr[0xB1] = OpCode::new(0xB1, "LDA", 2, 5, true, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lda(cpu, mode)));

        arr[0xA2] = OpCode::new(0xA2, "LDX", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ldx(cpu, mode)));
        arr[0xA6] = OpCode::new(0xA6, "LDX", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ldx(cpu, mode)));
        arr[0xB6] = OpCode::new(0xB6, "LDX", 2, 4, false, AddressingMode::ZeroPage_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ldx(cpu, mode)));
        arr[0xAE] = OpCode::new(0xAE, "LDX", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ldx(cpu, mode)));
        arr[0xBE] = OpCode::new(0xBE, "LDX", 3, 4, true, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ldx(cpu, mode)));

        arr[0xA7] = OpCode::new(0xA7, "*LAX", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lax(cpu, mode)));
        arr[0xB7] = OpCode::new(0xB7, "*LAX", 2, 4, false, AddressingMode::ZeroPage_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lax(cpu, mode)));
        arr[0xAF] = OpCode::new(0xAF, "*LAX", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lax(cpu, mode)));
        arr[0xBF] = OpCode::new(0xBF, "*LAX", 3, 4, true, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lax(cpu, mode)));
        arr[0xA3] = OpCode::new(0xA3, "*LAX", 2, 6, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lax(cpu, mode)));
        arr[0xB3] = OpCode::new(0xB3, "*LAX", 2, 5, true, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lax(cpu, mode)));

        arr[0xA0] = OpCode::new(0xA0, "LDY", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ldy(cpu, mode)));
        arr[0xA4] = OpCode::new(0xA4, "LDY", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ldy(cpu, mode)));
        arr[0xB4] = OpCode::new(0xB4, "LDY", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ldy(cpu, mode)));
        arr[0xAC] = OpCode::new(0xAC, "LDY", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ldy(cpu, mode)));
        arr[0xBC] = OpCode::new(0xBC, "LDY", 3, 4, true, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ldy(cpu, mode)));

        arr[0xAB] = OpCode::new(0xAB, "*ATX", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::atx(cpu, mode)));

        arr[0xBB] = OpCode::new(0xBB, "*LAR", 3, 4, true, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lar(cpu, mode)));

        /* Stores */
        arr[0x85] = OpCode::new(0x85, "STA", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sta(cpu, mode)));
        arr[0x95] = OpCode::new(0x95, "STA", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sta(cpu, mode)));
        arr[0x8D] = OpCode::new(0x8D, "STA", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sta(cpu, mode)));
        arr[0x9D] = OpCode::new(0x9D, "STA", 3, 5, false, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sta(cpu, mode)));
        arr[0x99] = OpCode::new(0x99, "STA", 3, 5, false, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sta(cpu, mode)));
        arr[0x81] = OpCode::new(0x81, "STA", 2, 6, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sta(cpu, mode)));
        arr[0x91] = OpCode::new(0x91, "STA", 2, 6, false, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sta(cpu, mode)));

        arr[0x86] = OpCode::new(0x86, "STX", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::stx(cpu, mode)));
        arr[0x96] = OpCode::new(0x96, "STX", 2, 4, false, AddressingMode::ZeroPage_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::stx(cpu, mode)));
        arr[0x8E] = OpCode::new(0x8E, "STX", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::stx(cpu, mode)));

        arr[0x84] = OpCode::new(0x84, "STY", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sty(cpu, mode)));
        arr[0x94] = OpCode::new(0x94, "STY", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sty(cpu, mode)));
        arr[0x8C] = OpCode::new(0x8C, "STY", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sty(cpu, mode)));

        /* Arithmetic */
        arr[0x69] = OpCode::new(0x69, "ADC", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::adc(cpu, mode)));
        arr[0x65] = OpCode::new(0x65, "ADC", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::adc(cpu, mode)));
        arr[0x75] = OpCode::new(0x75, "ADC", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::adc(cpu, mode)));
        arr[0x6D] = OpCode::new(0x6D, "ADC", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::adc(cpu, mode)));
        arr[0x7D] = OpCode::new(0x7D, "ADC", 3, 4, true, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::adc(cpu, mode)));
        arr[0x79] = OpCode::new(0x79, "ADC", 3, 4, true, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::adc(cpu, mode)));
        arr[0x61] = OpCode::new(0x61, "ADC", 2, 6, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::adc(cpu, mode)));
        arr[0x71] = OpCode::new(0x71, "ADC", 2, 5, true, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::adc(cpu, mode)));

        arr[0xE9] = OpCode::new(0xE9, "SBC", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sbc(cpu, mode)));
        arr[0xEB] = OpCode::new(0xEB, "*SBC", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sbc(cpu, mode)));
        arr[0xE5] = OpCode::new(0xE5, "SBC", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sbc(cpu, mode)));
        arr[0xF5] = OpCode::new(0xF5, "SBC", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sbc(cpu, mode)));
        arr[0xED] = OpCode::new(0xED, "SBC", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sbc(cpu, mode)));
        arr[0xFD] = OpCode::new(0xFD, "SBC", 3, 4, true, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sbc(cpu, mode)));
        arr[0xF9] = OpCode::new(0xF9, "SBC", 3, 4, true, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sbc(cpu, mode)));
        arr[0xE1] = OpCode::new(0xE1, "SBC", 2, 6, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sbc(cpu, mode)));
        arr[0xF1] = OpCode::new(0xF1, "SBC", 2, 5, true, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sbc(cpu, mode)));

        arr[0xC6] = OpCode::new(0xC6, "DEC", 2, 5, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::dec(cpu, mode)));
        arr[0xD6] = OpCode::new(0xD6, "DEC", 2, 6, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::dec(cpu, mode)));
        arr[0xCE] = OpCode::new(0xCE, "DEC", 3, 6, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::dec(cpu, mode)));
        arr[0xDE] = OpCode::new(0xDE, "DEC", 3, 7, false, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::dec(cpu, mode)));

        arr[0xCA] = OpCode::new(0xCA, "DEX", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::dex(cpu)));
        arr[0x88] = OpCode::new(0x88, "DEY", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::dey(cpu)));

        arr[0xE6] = OpCode::new(0xE6, "INC", 2, 5, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::inc(cpu, mode)));
        arr[0xF6] = OpCode::new(0xF6, "INC", 2, 6, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::inc(cpu, mode)));
        arr[0xEE] = OpCode::new(0xEE, "INC", 3, 6, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::inc(cpu, mode)));
        arr[0xFE] = OpCode::new(0xFE, "INC", 3, 7, false, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::inc(cpu, mode)));

        arr[0xE8] = OpCode::new(0xE8, "INX", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::inx(cpu)));
        arr[0xC8] = OpCode::new(0xC8, "INY", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::iny(cpu)));

        arr[0xC7] = OpCode::new(0xC7, "*DCP", 2, 5, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::dcp(cpu, mode)));
        arr[0xD7] = OpCode::new(0xD7, "*DCP", 2, 6, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::dcp(cpu, mode)));
        arr[0xCF] = OpCode::new(0xCF, "*DCP", 3, 6, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::dcp(cpu, mode)));
        arr[0xDF] = OpCode::new(0xDF, "*DCP", 3, 7, false, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::dcp(cpu, mode)));
        arr[0xDB] = OpCode::new(0xDB, "*DCP", 3, 7, false, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::dcp(cpu, mode)));
        arr[0xC3] = OpCode::new(0xC3, "*DCP", 2, 8, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::dcp(cpu, mode)));
        arr[0xD3] = OpCode::new(0xD3, "*DCP", 2, 8, false, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::dcp(cpu, mode)));

        arr[0xE7] = OpCode::new(0xE7, "*ISB", 2, 5, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::isb(cpu, mode)));
        arr[0xF7] = OpCode::new(0xF7, "*ISB", 2, 6, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::isb(cpu, mode)));
        arr[0xEF] = OpCode::new(0xEF, "*ISB", 3, 6, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::isb(cpu, mode)));
        arr[0xFF] = OpCode::new(0xFF, "*ISB", 3, 7, false, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::isb(cpu, mode)));
        arr[0xFB] = OpCode::new(0xFB, "*ISB", 3, 7, false, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::isb(cpu, mode)));
        arr[0xE3] = OpCode::new(0xE3, "*ISB", 2, 8, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::isb(cpu, mode)));
        arr[0xF3] = OpCode::new(0xF3, "*ISB", 2, 8, false, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::isb(cpu, mode)));

        arr[0xCB] = OpCode::new(0xCB, "*AXS", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::axs(cpu, mode)));

        /* Bit Operations */
        arr[0x29] = OpCode::new(0x29, "AND", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::and(cpu, mode)));
        arr[0x25] = OpCode::new(0x25, "AND", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::and(cpu, mode)));
        arr[0x35] = OpCode::new(0x35, "AND", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::and(cpu, mode)));
        arr[0x2D] = OpCode::new(0x2D, "AND", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::and(cpu, mode)));
        arr[0x3D] = OpCode::new(0x3D, "AND", 3, 4, true, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::and(cpu, mode)));
        arr[0x39] = OpCode::new(0x39, "AND", 3, 4, true, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::and(cpu, mode)));
        arr[0x21] = OpCode::new(0x21, "AND", 2, 6, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::and(cpu, mode)));
        arr[0x31] = OpCode::new(0x31, "AND", 2, 5, true, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::and(cpu, mode)));

        arr[0x49] = OpCode::new(0x49, "EOR", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::eor(cpu, mode)));
        arr[0x45] = OpCode::new(0x45, "EOR", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::eor(cpu, mode)));
        arr[0x55] = OpCode::new(0x55, "EOR", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::eor(cpu, mode)));
        arr[0x4D] = OpCode::new(0x4D, "EOR", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::eor(cpu, mode)));
        arr[0x5D] = OpCode::new(0x5D, "EOR", 3, 4, true, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::eor(cpu, mode)));
        arr[0x59] = OpCode::new(0x59, "EOR", 3, 4, true, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::eor(cpu, mode)));
        arr[0x41] = OpCode::new(0x41, "EOR", 2, 6, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::eor(cpu, mode)));
        arr[0x51] = OpCode::new(0x51, "EOR", 2, 5, true, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::eor(cpu, mode)));

        arr[0x09] = OpCode::new(0x09, "ORA", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ora(cpu, mode)));
        arr[0x05] = OpCode::new(0x05, "ORA", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ora(cpu, mode)));
        arr[0x15] = OpCode::new(0x15, "ORA", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ora(cpu, mode)));
        arr[0x0D] = OpCode::new(0x0D, "ORA", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ora(cpu, mode)));
        arr[0x1D] = OpCode::new(0x1D, "ORA", 3, 4, true, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ora(cpu, mode)));
        arr[0x19] = OpCode::new(0x19, "ORA", 3, 4, true, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ora(cpu, mode)));
        arr[0x01] = OpCode::new(0x01, "ORA", 2, 6, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ora(cpu, mode)));
        arr[0x11] = OpCode::new(0x11, "ORA", 2, 5, true, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ora(cpu, mode)));

        arr[0x0A] = OpCode::new(0x0A, "ASL", 1, 2, false, AddressingMode::Accumulator, Operation::FnCpuAndAddressing(|cpu, mode| CPU::asl(cpu, mode)));
        arr[0x06] = OpCode::new(0x06, "ASL", 2, 5, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::asl(cpu, mode)));
        arr[0x16] = OpCode::new(0x16, "ASL", 2, 6, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::asl(cpu, mode)));
        arr[0x0E] = OpCode::new(0x0E, "ASL", 3, 6, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::asl(cpu, mode)));
        arr[0x1E] = OpCode::new(0x1E, "ASL", 3, 7, false, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::asl(cpu, mode)));

        arr[0x4A] = OpCode::new(0x4A, "LSR", 1, 2, false, AddressingMode::Accumulator, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lsr(cpu, mode)));
        arr[0x46] = OpCode::new(0x46, "LSR", 2, 5, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lsr(cpu, mode)));
        arr[0x56] = OpCode::new(0x56, "LSR", 2, 6, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lsr(cpu, mode)));
        arr[0x4E] = OpCode::new(0x4E, "LSR", 3, 6, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lsr(cpu, mode)));
        arr[0x5E] = OpCode::new(0x5E, "LSR", 3, 7, false, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::lsr(cpu, mode)));

        arr[0x2A] = OpCode::new(0x2A, "ROL", 1, 2, false, AddressingMode::Accumulator, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rol(cpu, mode)));
        arr[0x26] = OpCode::new(0x26, "ROL", 2, 5, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rol(cpu, mode)));
        arr[0x36] = OpCode::new(0x36, "ROL", 2, 6, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rol(cpu, mode)));
        arr[0x2E] = OpCode::new(0x2E, "ROL", 3, 6, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rol(cpu, mode)));
        arr[0x3E] = OpCode::new(0x3E, "ROL", 3, 7, false, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rol(cpu, mode)));

        arr[0x6A] = OpCode::new(0x6A, "ROR", 1, 2, false, AddressingMode::Accumulator, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ror(cpu, mode)));
        arr[0x66] = OpCode::new(0x66, "ROR", 2, 5, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ror(cpu, mode)));
        arr[0x76] = OpCode::new(0x76, "ROR", 2, 6, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ror(cpu, mode)));
        arr[0x6E] = OpCode::new(0x6E, "ROR", 3, 6, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ror(cpu, mode)));
        arr[0x7E] = OpCode::new(0x7E, "ROR", 3, 7, false, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::ror(cpu, mode)));

        arr[0x87] = OpCode::new(0x87, "*SAX", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sax(cpu, mode)));
        arr[0x97] = OpCode::new(0x97, "*SAX", 2, 4, false, AddressingMode::ZeroPage_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sax(cpu, mode)));
        arr[0x83] = OpCode::new(0x83, "*SAX", 2, 6, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sax(cpu, mode)));
        arr[0x8F] = OpCode::new(0x8F, "*SAX", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sax(cpu, mode)));

        arr[0x07] = OpCode::new(0x07, "*SLO", 2, 5, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::slo(cpu, mode)));
        arr[0x17] = OpCode::new(0x17, "*SLO", 2, 6, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::slo(cpu, mode)));
        arr[0x0F] = OpCode::new(0x0F, "*SLO", 3, 6, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::slo(cpu, mode)));
        arr[0x1F] = OpCode::new(0x1F, "*SLO", 3, 7, false, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::slo(cpu, mode)));
        arr[0x1B] = OpCode::new(0x1B, "*SLO", 3, 7, false, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::slo(cpu, mode)));
        arr[0x03] = OpCode::new(0x03, "*SLO", 2, 8, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::slo(cpu, mode)));
        arr[0x13] = OpCode::new(0x13, "*SLO", 2, 8, false, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::slo(cpu, mode)));

        arr[0x27] = OpCode::new(0x27, "*RLA", 2, 5, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rla(cpu, mode)));
        arr[0x37] = OpCode::new(0x37, "*RLA", 2, 6, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rla(cpu, mode)));
        arr[0x2F] = OpCode::new(0x2F, "*RLA", 3, 6, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rla(cpu, mode)));
        arr[0x3F] = OpCode::new(0x3F, "*RLA", 3, 7, false, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rla(cpu, mode)));
        arr[0x3B] = OpCode::new(0x3B, "*RLA", 3, 7, false, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rla(cpu, mode)));
        arr[0x23] = OpCode::new(0x23, "*RLA", 2, 8, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rla(cpu, mode)));
        arr[0x33] = OpCode::new(0x33, "*RLA", 2, 8, false, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rla(cpu, mode)));

        arr[0x47] = OpCode::new(0x47, "*SRE", 2, 5, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sre(cpu, mode)));
        arr[0x57] = OpCode::new(0x57, "*SRE", 2, 6, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sre(cpu, mode)));
        arr[0x4F] = OpCode::new(0x4F, "*SRE", 3, 6, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sre(cpu, mode)));
        arr[0x5F] = OpCode::new(0x5F, "*SRE", 3, 7, false, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sre(cpu, mode)));
        arr[0x5B] = OpCode::new(0x5B, "*SRE", 3, 7, false, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sre(cpu, mode)));
        arr[0x43] = OpCode::new(0x43, "*SRE", 2, 8, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sre(cpu, mode)));
        arr[0x53] = OpCode::new(0x53, "*SRE", 2, 8, false, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sre(cpu, mode)));

        arr[0x67] = OpCode::new(0x67, "*RRA", 2, 5, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rra(cpu, mode)));
        arr[0x77] = OpCode::new(0x77, "*RRA", 2, 6, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rra(cpu, mode)));
        arr[0x6F] = OpCode::new(0x6F, "*RRA", 3, 6, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rra(cpu, mode)));
        arr[0x7F] = OpCode::new(0x7F, "*RRA", 3, 7, false, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rra(cpu, mode)));
        arr[0x7B] = OpCode::new(0x7B, "*RRA", 3, 7, false, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rra(cpu, mode)));
        arr[0x63] = OpCode::new(0x63, "*RRA", 2, 8, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rra(cpu, mode)));
        arr[0x73] = OpCode::new(0x73, "*RRA", 2, 8, false, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::rra(cpu, mode)));

        arr[0x0B] = OpCode::new(0x0B, "*AAC", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::aac(cpu, mode)));
        arr[0x2B] = OpCode::new(0x2B, "*AAC", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::aac(cpu, mode)));

        arr[0x4B] = OpCode::new(0x4B, "*ASR", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::asr(cpu, mode)));

        arr[0x6B] = OpCode::new(0x6B, "*ARR", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::arr(cpu, mode)));

        arr[0x8B] = OpCode::new(0x8B, "*XAA", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::xaa(cpu, mode)));

        arr[0x9F] = OpCode::new(0x9F, "*AXA", 3, 5, false, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::axa(cpu, mode)));
        arr[0x93] = OpCode::new(0x93, "*AXA", 2, 6, false, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::axa(cpu, mode)));

        arr[0x9B] = OpCode::new(0x9B, "*XAS", 3, 5, false, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::xas(cpu, mode)));

        arr[0x9C] = OpCode::new(0x9C, "*SYA", 3, 5, false, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sya(cpu, mode)));

        arr[0x9E] = OpCode::new(0x9E, "*SXA", 3, 5, false, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::sxa(cpu, mode)));

        /* Comparisons */
        arr[0x24] = OpCode::new(0x24, "BIT", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::bit(cpu, mode)));
        arr[0x2C] = OpCode::new(0x2C, "BIT", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::bit(cpu, mode)));

        arr[0xC9] = OpCode::new(0xC9, "CMP", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::cmp(cpu, mode)));
        arr[0xC5] = OpCode::new(0xC5, "CMP", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::cmp(cpu, mode)));
        arr[0xD5] = OpCode::new(0xD5, "CMP", 2, 4, false, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::cmp(cpu, mode)));
        arr[0xCD] = OpCode::new(0xCD, "CMP", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::cmp(cpu, mode)));
        arr[0xDD] = OpCode::new(0xDD, "CMP", 3, 4, true, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::cmp(cpu, mode)));
        arr[0xD9] = OpCode::new(0xD9, "CMP", 3, 4, true, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::cmp(cpu, mode)));
        arr[0xC1] = OpCode::new(0xC1, "CMP", 2, 6, false, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(|cpu, mode| CPU::cmp(cpu, mode)));
        arr[0xD1] = OpCode::new(0xD1, "CMP", 2, 5, true, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(|cpu, mode| CPU::cmp(cpu, mode)));

        arr[0xE0] = OpCode::new(0xE0, "CPX", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::cpx(cpu, mode)));
        arr[0xE4] = OpCode::new(0xE4, "CPX", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::cpx(cpu, mode)));
        arr[0xEC] = OpCode::new(0xEC, "CPX", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::cpx(cpu, mode)));

        arr[0xC0] = OpCode::new(0xC0, "CPY", 2, 2, false, AddressingMode::Immediate, Operation::FnCpuAndAddressing(|cpu, mode| CPU::cpy(cpu, mode)));
        arr[0xC4] = OpCode::new(0xC4, "CPY", 2, 3, false, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(|cpu, mode| CPU::cpy(cpu, mode)));
        arr[0xCC] = OpCode::new(0xCC, "CPY", 3, 4, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::cpy(cpu, mode)));

        /* Branches */
        arr[0x90] = OpCode::new(0x90, "BCC", 2, 2, false /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(|cpu, mode| CPU::bcc(cpu, mode)));
        arr[0xB0] = OpCode::new(0xB0, "BCS", 2, 2, false /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(|cpu, mode| CPU::bcs(cpu, mode)));
        arr[0xF0] = OpCode::new(0xF0, "BEQ", 2, 2, false /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(|cpu, mode| CPU::beq(cpu, mode)));
        arr[0xD0] = OpCode::new(0xD0, "BNE", 2, 2, false /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(|cpu, mode| CPU::bne(cpu, mode)));
        arr[0x10] = OpCode::new(0x10, "BPL", 2, 2, false /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(|cpu, mode| CPU::bpl(cpu, mode)));
        arr[0x30] = OpCode::new(0x30, "BMI", 2, 2, false /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(|cpu, mode| CPU::bmi(cpu, mode)));
        arr[0x50] = OpCode::new(0x50, "BVC", 2, 2, false /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(|cpu, mode| CPU::bvc(cpu, mode)));
        arr[0x70] = OpCode::new(0x70, "BVS", 2, 2, false /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(|cpu, mode| CPU::bvs(cpu, mode)));

        /* Jumps */
        arr[0x4C] = OpCode::new(0x4C, "JMP", 3, 3, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::jmp(cpu, mode)));
        arr[0x6C] = OpCode::new(0x6C, "JMP", 3, 5, false, AddressingMode::Indirect, Operation::FnCpuAndAddressing(|cpu, mode| CPU::jmp(cpu, mode)));

        arr[0x20] = OpCode::new(0x20, "JSR", 3, 6, false, AddressingMode::Absolute, Operation::FnCpuAndAddressing(|cpu, mode| CPU::jsr(cpu, mode)));

        /* Returns */
        arr[0x40] = OpCode::new(0x40, "RTI", 1, 6, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::rti(cpu)));
        arr[0x60] = OpCode::new(0x60, "RTS", 1, 6, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::rts(cpu)));

        /* Flags */
        arr[0x18] = OpCode::new(0x18, "CLC", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::clc(cpu)));
        arr[0xD8] = OpCode::new(0xD8, "CLD", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::cld(cpu)));
        arr[0x58] = OpCode::new(0x58, "CLI", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::cli(cpu)));
        arr[0xB8] = OpCode::new(0xB8, "CLV", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::clv(cpu)));

        arr[0x38] = OpCode::new(0x38, "SEC", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::sec(cpu)));
        arr[0xF8] = OpCode::new(0xF8, "SED", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::sed(cpu)));
        arr[0x78] = OpCode::new(0x78, "SEI", 1, 2, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::sei(cpu)));

        /* Stack */
        arr[0x48] = OpCode::new(0x48, "PHA", 1, 3, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::pha(cpu)));
        arr[0x08] = OpCode::new(0x08, "PHP", 1, 3, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::php(cpu)));
        arr[0x68] = OpCode::new(0x68, "PLA", 1, 4, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::pla(cpu)));
        arr[0x28] = OpCode::new(0x28, "PLP", 1, 4, false, AddressingMode::NonAddressing, Operation::FnCpu(|cpu| CPU::plp(cpu)));

        arr
    };
}

#[test]
fn test_size() {
    for opcode in CPU_INSTRUCTIONS.iter() {
        // brk is special
        if opcode.code == 0x00 {
            assert_eq!(opcode.size, 0);
            continue;
        }
        match opcode.mode {
            AddressingMode::Accumulator => assert_eq!(opcode.size, 1),
            AddressingMode::Immediate => assert_eq!(opcode.size, 2),
            AddressingMode::ZeroPage => assert_eq!(opcode.size, 2),
            AddressingMode::ZeroPage_X => assert_eq!(opcode.size, 2),
            AddressingMode::ZeroPage_Y => assert_eq!(opcode.size, 2),
            AddressingMode::Absolute => assert_eq!(opcode.size, 3),
            AddressingMode::Absolute_X => assert_eq!(opcode.size, 3),
            AddressingMode::Absolute_Y => assert_eq!(opcode.size, 3),
            AddressingMode::Indirect => assert_eq!(opcode.size, 3),
            AddressingMode::Indirect_X => assert_eq!(opcode.size, 2),
            AddressingMode::Indirect_Y => assert_eq!(opcode.size, 2),
            AddressingMode::Relative => assert_eq!(opcode.size, 2),
            AddressingMode::NonAddressing => assert_eq!(opcode.size, 1),
        }
    }
}

#[test]
fn test_for_duplicate_opcodes() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    for opcode in CPU_INSTRUCTIONS.iter().map(|opcode| opcode.code) {
        if set.contains(&opcode) {
            panic!("Duplicate opcode 0x{opcode:x}")
        } else {
            set.insert(opcode);
        }
    }
}

#[test]
fn test_for_missing_opcodes() {
    for i in 0..0xFF {
        if CPU_INSTRUCTIONS[i].mnemonics.is_empty() {
            println!("Instruction {i:02X} not found");
            panic!()
        }
    }
}

#[test]
fn test_opcode_matches_key() {
    for i in 0..0xFF {
        if CPU_INSTRUCTIONS[i].code != i as u8 {
            panic!(
                "Opcode 0x{:x} does not match key 0x{:x}",
                CPU_INSTRUCTIONS[i].code, i
            )
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OpCode {
    pub code: u8,
    pub mnemonics: &'static str,
    pub size: u16,
    pub cycles: u8,
    pub bonus_cycle_on_page_cross: bool,
    pub mode: AddressingMode,
    operation: Operation,
}

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    FnCpuAndAddressing(fn(&mut CPU, &AddressingMode)),
    FnCpu(fn(&mut CPU)),
    Fn(fn()),
}

impl OpCode {
    fn new(
        code: u8,
        name: &'static str,
        size: u16,
        cycles: u8,
        bonus_cycle_on_page_cross: bool,
        mode: AddressingMode,
        operation: Operation,
    ) -> Self {
        OpCode {
            code,
            mnemonics: name,
            size,
            cycles,
            mode,
            bonus_cycle_on_page_cross,
            operation,
        }
    }

    pub fn execute(&self, cpu: &mut CPU) {
        match self.operation {
            Operation::FnCpuAndAddressing(op) => op(cpu, &self.mode),
            Operation::FnCpu(op) => op(cpu),
            Operation::Fn(op) => op(),
        }
        cpu.program_counter = cpu.program_counter.wrapping_add(self.size);
    }
}
