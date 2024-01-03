use crate::cpu::opcodes::CPU_INSTRUCTIONS;

mod opcodes;
#[cfg(test)]
mod cpu_tests;

#[derive(Debug, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect,
    Indirect_X,
    Indirect_Y,
    Relative,
    NonAddressing,
}

#[repr(u8)]
enum Flags {
    Carry             = 0b0000_0001,
    Zero              = 0b0000_0010,
    InterruptDisabled = 0b0000_0100,
    DecimalMode       = 0b0000_1000,
    B                 = 0b0001_0000,
    One               = 0b0010_0000,
    Overflow          = 0b0100_0000,
    Negative          = 0b1000_0000,
}

#[allow(non_snake_case)]
pub struct CPU {
    register_a: u8,
    register_x: u8,
    register_y: u8,
    register_s: u8,
    status: u8,
    program_counter: u16,
    memory: [u8; 0xFFFF],
}

impl Default for CPU {
    fn default() -> Self {
        CPU::new()
    }
}

impl CPU {
    const STACK_BASE_ADDRESS: u16 = 0x0100;
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            register_s: 0xFF,
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
        self.register_s = 0xFF;
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
            AddressingMode::Indirect => self.mem_read_u16(self.mem_read_u16(self.program_counter)),
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
            AddressingMode::Relative => {
                let mut offset = self.mem_read(self.program_counter) as i8;
                if offset < 0 {
                   offset *= -1;
                    self.program_counter.wrapping_sub(offset as u16)
                } else {
                    self.program_counter + (offset as u16)
                }
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

    fn push(&mut self, data: u8) {
        self.mem_write(CPU::STACK_BASE_ADDRESS + self.register_s as u16, data);
        self.register_s = self.register_s.wrapping_sub(1);
    }

    fn pull(&mut self) -> u8 {
        let value = self.mem_read(CPU::STACK_BASE_ADDRESS + self.register_s.wrapping_add(1) as u16);
        self.register_s = self.register_s.wrapping_add(1);
        value
    }

    fn push_u16(&mut self, data: u16) {
        self.push((data >> 8) as u8);
        self.push((data & 0xFF) as u8);
        self.mem_write_u16(CPU::STACK_BASE_ADDRESS + self.register_s as u16 - 1, data);
        self.register_s = self.register_s.wrapping_sub(2);
    }

    fn pull_u16(&mut self) -> u16 {
        let lo = self.pull() as u16;
        let hi = self.pull() as u16;
        (hi << 8) | lo
    }

    fn get_flag(&self, flag: Flags) -> bool {
        self.status & flag as u8 != 0
    }

    fn clear_flag(&mut self, flag: Flags) {
        self.status &= !(flag as u8);
    }

    fn set_flag(&mut self, flag: Flags) {
        self.status |= flag as u8;
    }

    fn update_flag(&mut self, flag: Flags, set: bool) {
        if set {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }

    fn update_zero_and_negative_flags(&mut self, value: u8) {
        self.update_flag(Flags::Zero, value == 0);
        self.update_flag(Flags::Negative, value & 0b1000_0000 != 0);
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn nop() {}

    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let neg1 = self.register_a & 0b1000_0000 != 0;
        let neg2 = value & 0b1000_0000 != 0;
        let (result, carry1) = self.register_a.overflowing_add(value);
        let (result, carry2) = result.overflowing_add(self.get_flag(Flags::Carry) as u8);
        self.register_a = result;
        self.update_flag(Flags::Carry, carry1 | carry2);
        self.update_flag(Flags::Overflow, (neg1 & neg2 & (self.register_a & 0b1000_0000 == 0)) || (!neg1 & !neg2 & (self.register_a & 0b1000_0000 != 0)));
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a &= value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a ^= value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a |= value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        if *mode == AddressingMode::NonAddressing {
            let carry = self.register_a & 0b1000_0000 != 0;
            self.register_a = self.register_a.wrapping_shl(1);
            self.update_flag(Flags::Carry, carry);
            self.update_zero_and_negative_flags(self.register_a);
        } else {
            let addr = self.get_operand_address(mode);
            let mut value = self.mem_read(addr);
            let carry = value & 0b1000_0000 != 0;
            value = value.wrapping_shl(1);
            self.update_flag(Flags::Carry, carry);
            self.update_flag(Flags::Negative, value & 0b1000_0000 != 0);
            self.mem_write(addr, value);
        }
    }

    fn bcc(&mut self, mode: &AddressingMode) {
        if !self.get_flag(Flags::Carry) {
            self.program_counter = self.get_operand_address(mode);
        }
    }

    fn bcs(&mut self, mode: &AddressingMode) {
        if self.get_flag(Flags::Carry) {
            self.program_counter = self.get_operand_address(mode);
        }
    }

    fn beq(&mut self, mode: &AddressingMode) {
        if self.get_flag(Flags::Zero) {
            self.program_counter = self.get_operand_address(mode);
        }
    }

    fn bne(&mut self, mode: &AddressingMode) {
        if !self.get_flag(Flags::Zero) {
            self.program_counter = self.get_operand_address(mode);
        }
    }

    fn bpl(&mut self, mode: &AddressingMode) {
        if !self.get_flag(Flags::Negative) {
            self.program_counter = self.get_operand_address(mode);
        }
    }

    fn bmi(&mut self, mode: &AddressingMode) {
        if self.get_flag(Flags::Negative) {
            self.program_counter = self.get_operand_address(mode);
        }
    }

    fn bvc(&mut self, mode: &AddressingMode) {
        if !self.get_flag(Flags::Overflow) {
            self.program_counter = self.get_operand_address(mode);
        }
    }

    fn bvs(&mut self, mode: &AddressingMode) {
        if self.get_flag(Flags::Overflow) {
            self.program_counter = self.get_operand_address(mode);
        }
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.update_flag(Flags::Zero, value & self.register_a == 0);
        self.update_flag(Flags::Overflow, value & Flags::Overflow as u8 != 0);
        self.update_flag(Flags::Negative, value & Flags::Negative as u8 != 0);
    }

    fn clc(&mut self) {
        self.clear_flag(Flags::Carry);
    }

    fn cld(&mut self) {
        self.clear_flag(Flags::DecimalMode);
    }

    fn cli(&mut self) {
        self.clear_flag(Flags::InterruptDisabled);
    }

    fn clv(&mut self) {
        self.clear_flag(Flags::Overflow);
    }

    fn compare(&mut self, a: u8, b: u8) {
        self.update_flag(Flags::Carry, a >= b);
        self.update_zero_and_negative_flags(a.wrapping_sub(b));
    }

    fn cmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.compare(self.register_a, value);
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.compare(self.register_x, value);
    }

    fn cpy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.compare(self.register_y, value);
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr).wrapping_sub(1);
        self.update_zero_and_negative_flags(value);
        self.mem_write(addr, value);
    }

    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr).wrapping_add(1);
        self.update_zero_and_negative_flags(value);
        self.mem_write(addr, value);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn jmp(&mut self, mode: &AddressingMode) {
        self.program_counter = self.get_operand_address(mode).wrapping_sub(2); //-2 because after this call the program_counter will be incremented by 2 (the size of the instruction - 1)
    }

    fn jsr(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.push_u16(self.program_counter.wrapping_add(1)); // +1 because the program_counter was not incremented yet
        self.program_counter = addr.wrapping_sub(2); // -2 because the program_counter will be incremented by 2 (the size of the instruction - 1)
    }
}