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