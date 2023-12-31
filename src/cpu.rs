use std::collections::HashMap;
use lazy_static::lazy_static;

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

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NonAddressing,
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

    fn execute(&self, cpu: &mut CPU) {
        match self.operation {
            Operation::FnCpuAndAddressing(op) => op(cpu, &self.mode),
            Operation::FnCpu(op) => op(cpu),
            Operation::Fn(op) => op(),
        }
        cpu.program_counter += self.size-1;
    }
}

#[allow(non_snake_case)]
pub struct CPU {
    register_a: u8,
    register_x: u8,
    register_y: u8,
    status: u8,
    program_counter: u16,
    memory: [u8; 0xFFFF],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF],
        }
    }

    pub fn load_and_run(&mut self, program: &[u8]) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn load(&mut self, program: &[u8]) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(program);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn reset(&mut self) {
        self.register_x = 0;
        self.register_a = 0;
        self.register_y = 0;
        self.status = 0;
        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn run(&mut self) {
        loop {
            let op_code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            if op_code == 0 {
                return;
            }

            if let Some(instruction) = CPU_INSTRUCTIONS.get(&op_code) {
                instruction.execute(self);
            } else {
                panic!("Unknown opcode {:x}", op_code);
            }
        }
    }

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                pos.wrapping_add(self.register_x) as u16
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                pos.wrapping_add(self.register_y) as u16
            }
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                base.wrapping_add(self.register_x as u16)
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                base.wrapping_add(self.register_y as u16)
            }
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
                let ptr = base.wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                deref_base.wrapping_add(self.register_y as u16)
            }
            AddressingMode::NonAddressing => panic!("mode {:?} is not supported", mode),
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_read_u16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.mem_read(addr), self.mem_read(addr + 1)])
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn mem_write_u16(&mut self, addr: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xFF) as u8;
        self.mem_write(addr, lo);
        self.mem_write(addr + 1, hi);
    }


    fn update_zero_and_negative_flags(&mut self, value: u8) {
        if value == 0 {
            self.status |= 0b0000_0010;
        } else {
            self.status &= 0b1111_1101;
        }

        if value & 0b1000_0000 != 0 {
            self.status |= 0b1000_0000;
        } else {
            self.status &= 0b0111_1111;
        }
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn nop() {}
}

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