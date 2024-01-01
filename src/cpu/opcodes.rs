use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::cpu::{AddressingMode, CPU};

lazy_static! {
    pub static ref CPU_INSTRUCTIONS: HashMap<u8, OpCode> = {
        let mut map = HashMap::new();
        map.insert(0x00, OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NonAddressing, Operation::Fn(CPU::nop)));
        map.insert(0xAA, OpCode::new(0xAA, "TAX", 1, 2, AddressingMode::NonAddressing, Operation::FnCpu(CPU::tax)));
        map.insert(0xE8, OpCode::new(0xE8, "INX", 1, 2, AddressingMode::NonAddressing, Operation::FnCpu(CPU::inx)));
        map.insert(0xEA, OpCode::new(0x00, "NOP", 1, 2, AddressingMode::NonAddressing, Operation::Fn(CPU::nop)));

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

        map
    };
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