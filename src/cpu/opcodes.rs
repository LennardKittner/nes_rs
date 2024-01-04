use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::cpu::{AddressingMode, CPU};

lazy_static! {
    pub static ref CPU_INSTRUCTIONS: HashMap<u8, OpCode> = {
        let mut map = HashMap::new();
        //TODO: do more on BRK?
        map.insert(0x00, OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NonAddressing, Operation::Fn(CPU::nop)));
        map.insert(0xAA, OpCode::new(0xAA, "TAX", 1, 2, AddressingMode::NonAddressing, Operation::FnCpu(CPU::tax)));
        map.insert(0xEA, OpCode::new(0xEA, "NOP", 1, 2, AddressingMode::NonAddressing, Operation::Fn(CPU::nop)));

        map.insert(0xA9, OpCode::new(0xA9, "LDA", 2, 2, AddressingMode::Immediate, Operation::FnCpuAndAddressing(CPU::lda)));
        map.insert(0xA5, OpCode::new(0xA5, "LDA", 2, 3, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::lda)));
        map.insert(0xB5, OpCode::new(0xB5, "LDA", 2, 4, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(CPU::lda)));
        map.insert(0xAD, OpCode::new(0xAD, "LDA", 3, 4, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::lda)));
        map.insert(0xBD, OpCode::new(0xBD, "LDA", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(CPU::lda)));
        map.insert(0xB9, OpCode::new(0xB9, "LDA", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(CPU::lda)));
        map.insert(0xA1, OpCode::new(0xA1, "LDA", 2, 6, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(CPU::lda)));
        map.insert(0xB1, OpCode::new(0xB1, "LDA", 2, 5 /* +1 if page crossed */, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(CPU::lda)));

        map.insert(0x85, OpCode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::sta)));
        map.insert(0x95, OpCode::new(0x95, "STA", 2, 4, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(CPU::sta)));
        map.insert(0x8D, OpCode::new(0x8D, "STA", 3, 4, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::sta)));
        map.insert(0x9D, OpCode::new(0x9D, "STA", 3, 5, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(CPU::sta)));
        map.insert(0x99, OpCode::new(0x99, "STA", 3, 5, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(CPU::sta)));
        map.insert(0x81, OpCode::new(0x81, "STA", 2, 6, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(CPU::sta)));
        map.insert(0x91, OpCode::new(0x91, "STA", 2, 6, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(CPU::sta)));

        map.insert(0x69, OpCode::new(0x69, "ADC", 2, 2, AddressingMode::Immediate, Operation::FnCpuAndAddressing(CPU::adc)));
        map.insert(0x65, OpCode::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::adc)));
        map.insert(0x75, OpCode::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(CPU::adc)));
        map.insert(0x6D, OpCode::new(0x6D, "ADC", 3, 4, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::adc)));
        map.insert(0x7D, OpCode::new(0x7D, "ADC", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(CPU::adc)));
        map.insert(0x79, OpCode::new(0x79, "ADC", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(CPU::adc)));
        map.insert(0x61, OpCode::new(0x61, "ADC", 2, 6, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(CPU::adc)));
        map.insert(0x71, OpCode::new(0x71, "ADC", 2, 5 /* +1 if page crossed */, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(CPU::adc)));

        map.insert(0x29, OpCode::new(0x29, "AND", 2, 2, AddressingMode::Immediate, Operation::FnCpuAndAddressing(CPU::and)));
        map.insert(0x25, OpCode::new(0x25, "AND", 2, 3, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::and)));
        map.insert(0x35, OpCode::new(0x35, "AND", 2, 4, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(CPU::and)));
        map.insert(0x2D, OpCode::new(0x2D, "AND", 3, 4, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::and)));
        map.insert(0x3D, OpCode::new(0x3D, "AND", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(CPU::and)));
        map.insert(0x39, OpCode::new(0x39, "AND", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(CPU::and)));
        map.insert(0x21, OpCode::new(0x21, "AND", 2, 6, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(CPU::and)));
        map.insert(0x31, OpCode::new(0x31, "AND", 2, 5 /* +1 if page crossed */, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(CPU::and)));

        map.insert(0x49, OpCode::new(0x49, "EOR", 2, 2, AddressingMode::Immediate, Operation::FnCpuAndAddressing(CPU::eor)));
        map.insert(0x45, OpCode::new(0x45, "EOR", 2, 3, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::eor)));
        map.insert(0x55, OpCode::new(0x55, "EOR", 2, 4, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(CPU::eor)));
        map.insert(0x4D, OpCode::new(0x4D, "EOR", 3, 4, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::eor)));
        map.insert(0x5D, OpCode::new(0x5D, "EOR", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(CPU::eor)));
        map.insert(0x59, OpCode::new(0x59, "EOR", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(CPU::eor)));
        map.insert(0x41, OpCode::new(0x41, "EOR", 2, 6, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(CPU::eor)));
        map.insert(0x51, OpCode::new(0x51, "EOR", 2, 5 /* +1 if page crossed */, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(CPU::eor)));

        map.insert(0x09, OpCode::new(0x09, "ORA", 2, 2, AddressingMode::Immediate, Operation::FnCpuAndAddressing(CPU::ora)));
        map.insert(0x05, OpCode::new(0x05, "ORA", 2, 3, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::ora)));
        map.insert(0x15, OpCode::new(0x15, "ORA", 2, 4, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(CPU::ora)));
        map.insert(0x0D, OpCode::new(0x0D, "ORA", 3, 4, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::ora)));
        map.insert(0x1D, OpCode::new(0x1D, "ORA", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(CPU::ora)));
        map.insert(0x19, OpCode::new(0x19, "ORA", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(CPU::ora)));
        map.insert(0x01, OpCode::new(0x01, "ORA", 2, 6, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(CPU::ora)));
        map.insert(0x11, OpCode::new(0x11, "ORA", 2, 5 /* +1 if page crossed */, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(CPU::ora)));

        map.insert(0x0A, OpCode::new(0x0A, "ASL", 1, 2, AddressingMode::NonAddressing, Operation::FnCpuAndAddressing(CPU::asl)));
        map.insert(0x06, OpCode::new(0x06, "ASL", 2, 5, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::asl)));
        map.insert(0x16, OpCode::new(0x16, "ASL", 2, 6, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(CPU::asl)));
        map.insert(0x0E, OpCode::new(0x0E, "ASL", 3, 6, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::asl)));
        map.insert(0x1E, OpCode::new(0x1E, "ASL", 3, 7, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(CPU::asl)));

        map.insert(0x90, OpCode::new(0x90, "BCC", 2, 2 /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(CPU::bcc)));
        map.insert(0xB0, OpCode::new(0xB0, "BCS", 2, 2 /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(CPU::bcs)));
        map.insert(0xF0, OpCode::new(0xF0, "BEQ", 2, 2 /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(CPU::beq)));
        map.insert(0xD0, OpCode::new(0xD0, "BNE", 2, 2 /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(CPU::bne)));
        map.insert(0x10, OpCode::new(0x10, "BPL", 2, 2 /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(CPU::bpl)));
        map.insert(0x30, OpCode::new(0x30, "BMI", 2, 2 /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(CPU::bmi)));
        map.insert(0x50, OpCode::new(0x50, "BVC", 2, 2 /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(CPU::bvc)));
        map.insert(0x70, OpCode::new(0x70, "BVS", 2, 2 /* +1 if branch succeeds +2 if to a new page */, AddressingMode::Relative, Operation::FnCpuAndAddressing(CPU::bvs)));

        map.insert(0x24, OpCode::new(0x24, "BIT", 2, 3, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::bit)));
        map.insert(0x2C, OpCode::new(0x2C, "BIT", 3, 4, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::bit)));

        map.insert(0x18, OpCode::new(0x18, "CLC", 1, 2, AddressingMode::NonAddressing, Operation::FnCpu(CPU::clc)));
        map.insert(0xD8, OpCode::new(0xD8, "CLD", 1, 2, AddressingMode::NonAddressing, Operation::FnCpu(CPU::cld)));
        map.insert(0x58, OpCode::new(0x58, "CLI", 1, 2, AddressingMode::NonAddressing, Operation::FnCpu(CPU::cli)));
        map.insert(0xB8, OpCode::new(0xB8, "CLV", 1, 2, AddressingMode::NonAddressing, Operation::FnCpu(CPU::clv)));

        map.insert(0xC9, OpCode::new(0xC9, "CMP", 2, 2, AddressingMode::Immediate, Operation::FnCpuAndAddressing(CPU::cmp)));
        map.insert(0xC5, OpCode::new(0xC5, "CMP", 2, 3, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::cmp)));
        map.insert(0xD5, OpCode::new(0xD5, "CMP", 2, 4, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(CPU::cmp)));
        map.insert(0xCD, OpCode::new(0xCD, "CMP", 3, 4, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::cmp)));
        map.insert(0xDD, OpCode::new(0xDD, "CMP", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(CPU::cmp)));
        map.insert(0xD9, OpCode::new(0xD9, "CMP", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(CPU::cmp)));
        map.insert(0xC1, OpCode::new(0xC1, "CMP", 2, 6, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(CPU::cmp)));
        map.insert(0xD1, OpCode::new(0xD1, "CMP", 2, 5 /* +1 if page crossed */, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(CPU::cmp)));

        map.insert(0xE0, OpCode::new(0xE0, "CPX", 2, 2, AddressingMode::Immediate, Operation::FnCpuAndAddressing(CPU::cpx)));
        map.insert(0xE4, OpCode::new(0xE4, "CPX", 2, 3, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::cpx)));
        map.insert(0xEC, OpCode::new(0xEC, "CPX", 3, 4, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::cpx)));

        map.insert(0xC0, OpCode::new(0xC0, "CPY", 2, 2, AddressingMode::Immediate, Operation::FnCpuAndAddressing(CPU::cpy)));
        map.insert(0xC4, OpCode::new(0xC4, "CPY", 2, 3, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::cpy)));
        map.insert(0xCC, OpCode::new(0xCC, "CPY", 3, 4, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::cpy)));

        map.insert(0xC6, OpCode::new(0xC6, "DEC", 2, 5, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::dec)));
        map.insert(0xD6, OpCode::new(0xD6, "DEC", 2, 6, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(CPU::dec)));
        map.insert(0xCE, OpCode::new(0xCE, "DEC", 3, 6, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::dec)));
        map.insert(0xDE, OpCode::new(0xDE, "DEC", 3, 7, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(CPU::dec)));

        map.insert(0xCA, OpCode::new(0xCA, "DEX", 1, 2, AddressingMode::NonAddressing, Operation::FnCpu(CPU::dex)));
        map.insert(0x88, OpCode::new(0x88, "DEY", 1, 2, AddressingMode::NonAddressing, Operation::FnCpu(CPU::dey)));

        map.insert(0xE6, OpCode::new(0xE6, "INC", 2, 5, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::inc)));
        map.insert(0xF6, OpCode::new(0xF6, "INC", 2, 6, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(CPU::inc)));
        map.insert(0xEE, OpCode::new(0xEE, "INC", 3, 6, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::inc)));
        map.insert(0xFE, OpCode::new(0xFE, "INC", 3, 7, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(CPU::inc)));

        map.insert(0xE8, OpCode::new(0xE8, "INX", 1, 2, AddressingMode::NonAddressing, Operation::FnCpu(CPU::inx)));
        map.insert(0xC8, OpCode::new(0xC8, "INY", 1, 2, AddressingMode::NonAddressing, Operation::FnCpu(CPU::iny)));

        map.insert(0x4C, OpCode::new(0x4C, "JMP", 3, 3, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::jmp)));
        map.insert(0x6C, OpCode::new(0x6C, "JMP", 3, 5, AddressingMode::Indirect, Operation::FnCpuAndAddressing(CPU::jmp)));

        map.insert(0x20, OpCode::new(0x20, "JSR", 3, 6, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::jsr)));

        map.insert(0xA2, OpCode::new(0xA2, "LDX", 2, 2, AddressingMode::Immediate, Operation::FnCpuAndAddressing(CPU::ldx)));
        map.insert(0xA6, OpCode::new(0xA6, "LDX", 2, 3, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::ldx)));
        map.insert(0xB6, OpCode::new(0xB6, "LDX", 2, 4, AddressingMode::ZeroPage_Y, Operation::FnCpuAndAddressing(CPU::ldx)));
        map.insert(0xAE, OpCode::new(0xAE, "LDX", 3, 4, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::ldx)));
        map.insert(0xBE, OpCode::new(0xBE, "LDX", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(CPU::ldx)));

        map.insert(0xA0, OpCode::new(0xA0, "LDY", 2, 2, AddressingMode::Immediate, Operation::FnCpuAndAddressing(CPU::ldy)));
        map.insert(0xA4, OpCode::new(0xA4, "LDY", 2, 3, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::ldy)));
        map.insert(0xB4, OpCode::new(0xB4, "LDY", 2, 4, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(CPU::ldy)));
        map.insert(0xAC, OpCode::new(0xAC, "LDY", 3, 4, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::ldy)));
        map.insert(0xBC, OpCode::new(0xBC, "LDY", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(CPU::ldy)));

        map.insert(0x4A, OpCode::new(0x4A, "LSR", 1, 2, AddressingMode::NonAddressing, Operation::FnCpuAndAddressing(CPU::lsr)));
        map.insert(0x46, OpCode::new(0x46, "LSR", 2, 5, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::lsr)));
        map.insert(0x56, OpCode::new(0x56, "LSR", 2, 6, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(CPU::lsr)));
        map.insert(0x4E, OpCode::new(0x4E, "LSR", 3, 6, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::lsr)));
        map.insert(0x5E, OpCode::new(0x5E, "LSR", 3, 7, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(CPU::lsr)));

        map.insert(0x48, OpCode::new(0x48, "PHA", 1, 3, AddressingMode::NonAddressing, Operation::FnCpu(CPU::pha)));
        map.insert(0x08, OpCode::new(0x08, "PHP", 1, 3, AddressingMode::NonAddressing, Operation::FnCpu(CPU::php)));
        map.insert(0x68, OpCode::new(0x68, "PLA", 1, 4, AddressingMode::NonAddressing, Operation::FnCpu(CPU::pla)));
        map.insert(0x28, OpCode::new(0x28, "PLP", 1, 4, AddressingMode::NonAddressing, Operation::FnCpu(CPU::plp)));

        map.insert(0x2A, OpCode::new(0x2A, "ROL", 1, 2, AddressingMode::NonAddressing, Operation::FnCpuAndAddressing(CPU::rol)));
        map.insert(0x26, OpCode::new(0x26, "ROL", 2, 5, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::rol)));
        map.insert(0x36, OpCode::new(0x36, "ROL", 2, 6, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(CPU::rol)));
        map.insert(0x2E, OpCode::new(0x2E, "ROL", 3, 6, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::rol)));
        map.insert(0x3E, OpCode::new(0x3E, "ROL", 3, 7, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(CPU::rol)));

        map.insert(0x6A, OpCode::new(0x6A, "ROR", 1, 2, AddressingMode::NonAddressing, Operation::FnCpuAndAddressing(CPU::ror)));
        map.insert(0x66, OpCode::new(0x66, "ROR", 2, 5, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::ror)));
        map.insert(0x76, OpCode::new(0x76, "ROR", 2, 6, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(CPU::ror)));
        map.insert(0x6E, OpCode::new(0x6E, "ROR", 3, 6, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::ror)));
        map.insert(0x7E, OpCode::new(0x7E, "ROR", 3, 7, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(CPU::ror)));

        map.insert(0x40, OpCode::new(0x40, "RTI", 1, 6, AddressingMode::NonAddressing, Operation::FnCpu(CPU::rti)));
        map.insert(0x60, OpCode::new(0x60, "RTS", 1, 6, AddressingMode::NonAddressing, Operation::FnCpu(CPU::rts)));

        map.insert(0xE9, OpCode::new(0xE9, "SBC", 2, 2, AddressingMode::Immediate, Operation::FnCpuAndAddressing(CPU::sbc)));
        map.insert(0xE5, OpCode::new(0xE5, "SBC", 2, 3, AddressingMode::ZeroPage, Operation::FnCpuAndAddressing(CPU::sbc)));
        map.insert(0xF5, OpCode::new(0xF5, "SBC", 2, 4, AddressingMode::ZeroPage_X, Operation::FnCpuAndAddressing(CPU::sbc)));
        map.insert(0xED, OpCode::new(0xED, "SBC", 3, 4, AddressingMode::Absolute, Operation::FnCpuAndAddressing(CPU::sbc)));
        map.insert(0xFD, OpCode::new(0xFD, "SBC", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_X, Operation::FnCpuAndAddressing(CPU::sbc)));
        map.insert(0xF9, OpCode::new(0xF9, "SBC", 3, 4 /* +1 if page crossed */, AddressingMode::Absolute_Y, Operation::FnCpuAndAddressing(CPU::sbc)));
        map.insert(0xE1, OpCode::new(0xE1, "SBC", 2, 6, AddressingMode::Indirect_X, Operation::FnCpuAndAddressing(CPU::sbc)));
        map.insert(0xF1, OpCode::new(0xF1, "SBC", 2, 5 /* +1 if page crossed */, AddressingMode::Indirect_Y, Operation::FnCpuAndAddressing(CPU::sbc)));

        map
    };
}

#[test]
fn test_size() {
    use std::collections::HashSet;
    for opcode in CPU_INSTRUCTIONS.values() {
        match opcode.mode {
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
    for opcode in CPU_INSTRUCTIONS.values().into_iter().map(|oc| oc.code) {
        if set.contains(&opcode) {
            panic!("Duplicate opcode 0x{opcode:x}")
        } else {
            set.insert(opcode);
        }
    }
}

#[test]
fn test_opcode_matches_key() {
    for (key, opcode) in CPU_INSTRUCTIONS.iter().map(|(i, oc)| (i, oc.code)) {
        if *key != opcode {
            panic!("Opcode 0x{opcode:x} does not match key 0x{key:x}")
        }
    }
}

#[allow(dead_code)]
pub struct OpCode {
    code: u8,
    name: &'static str,
    size: u16,
    cycles: u8,
    mode: AddressingMode,
    operation: Operation
}

pub enum Operation {
    FnCpuAndAddressing(fn(&mut CPU, &AddressingMode)),
    FnCpu(fn(&mut CPU)),
    Fn(fn()),
}

impl OpCode {
    fn new(code: u8, name: &'static str, size: u16, cycles: u8, mode: AddressingMode, operation: Operation) -> Self {
        OpCode {
            code,
            name,
            size,
            cycles,
            mode,
            operation,
        }
    }

    pub fn execute(&self, cpu: &mut CPU) {
        match self.operation {
            Operation::FnCpuAndAddressing(op) => op(cpu, &self.mode),
            Operation::FnCpu(op) => op(cpu),
            Operation::Fn(op) => op(),
        }
        cpu.program_counter += self.size-1;
    }
}