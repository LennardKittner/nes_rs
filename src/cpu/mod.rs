use crate::bus::{Bus, Mem, PollIRQ, PollNMI};
use crate::cpu::addressing_mode::{page_cross, AddressingMode};
use crate::cpu::interrupts::{Interrupt, IRQ_INTERRUPT, NMI_INTERRUPT, RESET_INTERRUPT};
use crate::cpu::opcodes::CPU_INSTRUCTIONS;
use crate::ppu::palette::SystemPalette;
use crate::rom::Rom;

pub mod addressing_mode;
#[cfg(test)]
mod cpu_tests;
mod interrupts;
pub mod opcodes;

// 7  bit  0
// ---- ----
// NV1B DIZC
// |||| ||||
// |||| |||+- Carry
// |||| ||+-- Zero
// |||| |+--- Interrupt Disable
// |||| +---- Decimal
// |||+------ (No CPU effect; see: the B flag)
// ||+------- (No CPU effect; always pushed as 1)
// |+-------- Overflow
// +--------- Negative
#[derive(Copy, Clone)]
#[repr(u8)]
enum Flags {
    Carry = 0b0000_0001,
    Zero = 0b0000_0010,
    InterruptDisabled = 0b0000_0100,
    DecimalMode = 0b0000_1000,
    B1 = 0b0001_0000,
    B2 = 0b0010_0000,
    Overflow = 0b0100_0000,
    Negative = 0b1000_0000,
}

#[allow(non_snake_case)]
pub struct CPU<'a> {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub register_s: u8,
    pub status: u8,
    pub program_counter: u16,
    pub bus: Bus<'a>,
    // indicates how many additional cycles the previous instruction took
    additional_cycles: u8,
}

impl Mem for CPU<'_> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data);
    }
}

impl CPU<'_> {
    const STACK_BASE_ADDRESS: u16 = 0x0100;
    const STACK_END: u8 = 0xFD;
    const INITIAL_STATUS: u8 = 0x24;

    pub fn new(rom: Rom) -> Self {
        CPU::new_with_bus(Bus::new(rom, SystemPalette::new(), |_, _, _| {}, |_, _| {}))
    }

    pub fn new_with_bus(bus: Bus) -> CPU {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            register_s: CPU::STACK_END,
            status: CPU::INITIAL_STATUS,
            program_counter: 0,
            bus,
            additional_cycles: 0,
        }
    }

    pub fn load_and_run(&mut self, program: &[u8], at: u16) {
        self.load(program, at);
        self.reset();
        self.run();
    }

    pub fn load(&mut self, program: &[u8], at: u16) {
        for (i, byte) in program.iter().enumerate() {
            self.mem_write(at + i as u16, *byte);
        }
        self.mem_write_u16(RESET_INTERRUPT.interrupt_vector, at);
    }

    pub fn reset(&mut self) {
        self.register_x = 0;
        self.register_a = 0;
        self.register_y = 0;
        self.register_s = CPU::STACK_END;
        self.status = CPU::INITIAL_STATUS;
        self.program_counter = self.mem_read_u16(RESET_INTERRUPT.interrupt_vector);
        self.bus.tick(RESET_INTERRUPT.cycles);
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        loop {
            if self.bus.poll_nmi_status() {
                self.handle_interrupt(NMI_INTERRUPT);
            }
            if self.bus.poll_irq() {
                self.handle_interrupt(IRQ_INTERRUPT);
            }

            callback(self);
            let op_code = self.mem_read(self.program_counter);

            let instruction = CPU_INSTRUCTIONS[op_code as usize];
            instruction.execute(self);
            if op_code == 0x00 {
                return;
            }

            self.bus.tick(instruction.cycles + self.additional_cycles);
            self.additional_cycles = 0;
        }
    }

    fn handle_interrupt(&mut self, interrupt: Interrupt) {
        self.push_u16(self.program_counter);
        let mut status = self.status | Flags::B2 as u8;
        if interrupt.b_flag {
            status |= Flags::B1 as u8;
        }
        self.push(status);
        self.status |= Flags::InterruptDisabled as u8;
        self.program_counter = self.mem_read_u16(interrupt.interrupt_vector);
        self.bus.tick(interrupt.cycles);
    }

    fn get_operand(&mut self, mode: &AddressingMode) -> u8 {
        let addr = mode.get_operand_address(self).unwrap();
        self.mem_read(addr)
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

    fn branch(&mut self, condition: bool, target: u16) {
        if condition {
            self.additional_cycles += 1 + if page_cross(self.program_counter, target) {
                1
            } else {
                0
            };
            self.program_counter = target;
        }
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);

        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);

        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = mode.get_operand_address(self).unwrap();
        self.mem_write(addr, self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn tsx(&mut self) {
        self.register_x = self.register_s;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn txs(&mut self) {
        self.register_s = self.register_x;
    }

    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn nop() {}

    fn brk(&mut self) {
        //TODO: maybe pc +2 before handling interrupt
        //TODO: just exit for now
        //self.handle_interrupt(BRK_INTERRUPT);
    }

    fn add_to_a(&mut self, value: u8) {
        let neg1 = self.register_a & 0b1000_0000 != 0;
        let neg2 = value & 0b1000_0000 != 0;
        let (result, carry1) = self.register_a.overflowing_add(value);
        let (result, carry2) = result.overflowing_add(self.get_flag(Flags::Carry) as u8);
        self.register_a = result;
        self.update_flag(Flags::Carry, carry1 | carry2);
        self.update_flag(
            Flags::Overflow,
            (neg1 & neg2 & (self.register_a & 0b1000_0000 == 0))
                || (!neg1 & !neg2 & (self.register_a & 0b1000_0000 != 0)),
        );
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
        self.add_to_a(value);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
        self.register_a &= value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
        self.register_a ^= value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
        self.register_a |= value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn asl_logic(&mut self, mode: &AddressingMode) -> u8 {
        let mut addr = 0;
        let mut value = if *mode == AddressingMode::Accumulator {
            self.register_a
        } else {
            addr = mode.get_operand_address(self).unwrap();
            self.mem_read(addr)
        };
        let carry = value & 0b1000_0000 != 0;
        value = value.wrapping_shl(1);
        self.update_flag(Flags::Carry, carry);
        self.update_zero_and_negative_flags(value);
        if *mode == AddressingMode::Accumulator {
            self.register_a = value;
        } else {
            self.mem_write(addr, value);
        }
        value
    }

    fn asl(&mut self, mode: &AddressingMode) {
        let _ = self.asl_logic(mode);
    }

    fn bcc(&mut self, mode: &AddressingMode) {
        let target = mode.get_operand_address(self).unwrap();
        self.branch(!self.get_flag(Flags::Carry), target);
    }

    fn bcs(&mut self, mode: &AddressingMode) {
        let target = mode.get_operand_address(self).unwrap();
        self.branch(self.get_flag(Flags::Carry), target);
    }

    fn beq(&mut self, mode: &AddressingMode) {
        let target = mode.get_operand_address(self).unwrap();
        self.branch(self.get_flag(Flags::Zero), target);
    }

    fn bne(&mut self, mode: &AddressingMode) {
        let target = mode.get_operand_address(self).unwrap();
        self.branch(!self.get_flag(Flags::Zero), target);
    }

    fn bpl(&mut self, mode: &AddressingMode) {
        let target = mode.get_operand_address(self).unwrap();
        self.branch(!self.get_flag(Flags::Negative), target);
    }

    fn bmi(&mut self, mode: &AddressingMode) {
        let target = mode.get_operand_address(self).unwrap();
        self.branch(self.get_flag(Flags::Negative), target);
    }

    fn bvc(&mut self, mode: &AddressingMode) {
        let target = mode.get_operand_address(self).unwrap();
        self.branch(!self.get_flag(Flags::Overflow), target);
    }

    fn bvs(&mut self, mode: &AddressingMode) {
        let target = mode.get_operand_address(self).unwrap();
        self.branch(self.get_flag(Flags::Overflow), target);
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
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
        let value = self.get_operand(mode);
        self.compare(self.register_a, value);
    }

    fn cpx(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
        self.compare(self.register_x, value);
    }

    fn cpy(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
        self.compare(self.register_y, value);
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let addr = mode.get_operand_address(self).unwrap();
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
        let addr = mode.get_operand_address(self).unwrap();
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
        self.program_counter = mode
            .get_operand_address(self)
            .unwrap()
            .wrapping_sub(CPU_INSTRUCTIONS[0x4C].size); //because after this call the program_counter will be incremented by the size of the instruction
    }

    fn jsr(&mut self, mode: &AddressingMode) {
        let addr = mode.get_operand_address(self).unwrap();
        self.push_u16(self.program_counter.wrapping_add(2)); // +2 because the program_counter was not incremented yet
        self.program_counter = addr.wrapping_sub(CPU_INSTRUCTIONS[0x20].size); // because the program_counter will be incremented by the size of the instruction
    }

    fn lsr_logic(&mut self, mode: &AddressingMode) -> u8 {
        let mut addr = 0;
        let mut value = if *mode == AddressingMode::Accumulator {
            self.register_a
        } else {
            addr = mode.get_operand_address(self).unwrap();
            self.mem_read(addr)
        };
        let carry = value & 0b0000_0001 != 0;
        self.update_flag(Flags::Carry, carry);
        value = value.wrapping_shr(1);
        self.update_flag(Flags::Carry, carry);
        self.update_zero_and_negative_flags(value);
        if *mode == AddressingMode::Accumulator {
            self.register_a = value;
        } else {
            self.mem_write(addr, value);
        }
        value
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        let _ = self.lsr_logic(mode);
    }

    fn pha(&mut self) {
        self.push(self.register_a);
    }

    fn php(&mut self) {
        self.push(self.status | Flags::B1 as u8 | Flags::B2 as u8);
    }

    fn pla(&mut self) {
        self.register_a = self.pull();
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn plp(&mut self) {
        self.status = self.pull();
        self.set_flag(Flags::B2);
        self.clear_flag(Flags::B1);
    }

    fn rol_logic(&mut self, mode: &AddressingMode) -> u8 {
        if *mode == AddressingMode::Accumulator {
            let carry = self.register_a & 0b1000_0000 != 0;
            self.register_a = self.register_a.wrapping_shl(1);
            self.register_a |= if self.get_flag(Flags::Carry) { 1 } else { 0 };
            self.update_flag(Flags::Carry, carry);
            self.update_zero_and_negative_flags(self.register_a);
            self.register_a
        } else {
            let addr = mode.get_operand_address(self).unwrap();
            let mut value = self.mem_read(addr);
            let carry = value & 0b1000_0000 != 0;
            value = value.wrapping_shl(1);
            value |= if self.get_flag(Flags::Carry) { 1 } else { 0 };
            self.update_flag(Flags::Carry, carry);
            self.update_flag(Flags::Negative, value & 0b1000_0000 != 0);
            self.mem_write(addr, value);
            value
        }
    }

    fn rol(&mut self, mode: &AddressingMode) {
        let _ = self.rol_logic(mode);
    }

    fn ror_logic(&mut self, mode: &AddressingMode) -> u8 {
        if *mode == AddressingMode::Accumulator {
            let carry = self.register_a & 0b0000_0001 != 0;
            self.register_a = self.register_a.wrapping_shr(1);
            self.register_a |= if self.get_flag(Flags::Carry) {
                0b1000_0000
            } else {
                0
            };
            self.update_flag(Flags::Carry, carry);
            self.update_zero_and_negative_flags(self.register_a);
            self.register_a
        } else {
            let addr = mode.get_operand_address(self).unwrap();
            let mut value = self.mem_read(addr);
            let carry = value & 0b0000_0001 != 0;
            value = value.wrapping_shr(1);
            value |= if self.get_flag(Flags::Carry) {
                0b1000_0000
            } else {
                0
            };
            self.update_flag(Flags::Carry, carry);
            self.update_flag(Flags::Negative, value & 0b1000_0000 != 0);
            self.mem_write(addr, value);
            value
        }
    }

    fn ror(&mut self, mode: &AddressingMode) {
        let _ = self.ror_logic(mode);
    }

    fn rti(&mut self) {
        self.plp();
        self.program_counter = self.pull_u16().wrapping_sub(1); // subtract the size of rti
    }

    fn rts(&mut self) {
        self.program_counter = self.pull_u16();
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
        // The value has to be bitwise negated instead of arithmetically negated because of the carry
        self.add_to_a(!value);
    }

    fn sec(&mut self) {
        self.set_flag(Flags::Carry);
    }

    fn sed(&mut self) {
        self.set_flag(Flags::DecimalMode);
    }

    fn sei(&mut self) {
        self.set_flag(Flags::InterruptDisabled);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let addr = mode.get_operand_address(self).unwrap();
        self.mem_write(addr, self.register_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let addr = mode.get_operand_address(self).unwrap();
        self.mem_write(addr, self.register_y);
    }

    fn lax(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
        self.register_a = value;
        self.register_x = value;
        self.update_zero_and_negative_flags(value)
    }

    fn sax(&mut self, mode: &AddressingMode) {
        let addr = mode.get_operand_address(self).unwrap();
        let value = self.register_a & self.register_x;
        self.mem_write(addr, value);
    }

    fn dcp(&mut self, mode: &AddressingMode) {
        let addr = mode.get_operand_address(self).unwrap();
        let value = self.mem_read(addr).wrapping_sub(1);
        self.mem_write(addr, value);
        self.compare(self.register_a, value);
    }

    fn isb(&mut self, mode: &AddressingMode) {
        let addr = mode.get_operand_address(self).unwrap();
        let value = self.mem_read(addr).wrapping_add(1);
        self.mem_write(addr, value);
        // The value has to be bitwise negated instead of arithmetically negated because of the carry
        self.add_to_a(!value);
    }

    fn slo(&mut self, mode: &AddressingMode) {
        let value = self.asl_logic(mode);
        self.register_a |= value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn rla(&mut self, mode: &AddressingMode) {
        let value = self.rol_logic(mode);
        self.register_a &= value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn sre(&mut self, mode: &AddressingMode) {
        let value = self.lsr_logic(mode);
        self.register_a ^= value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn rra(&mut self, mode: &AddressingMode) {
        let value = self.ror_logic(mode);
        self.add_to_a(value);
    }

    fn aac(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
        self.register_a &= value;
        self.update_flag(Flags::Carry, 0b1000_0000 & self.register_a != 0);
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn kil(&mut self) {
        self.program_counter -= 1;
    }

    fn asr(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
        self.register_a &= value;
        self.lsr_logic(&AddressingMode::Accumulator);
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn arr(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
        self.register_a &= value;
        self.ror_logic(&AddressingMode::Accumulator);
        self.update_zero_and_negative_flags(self.register_a);
    }

    // Exact operation unknown
    fn xaa(&mut self, mode: &AddressingMode) {
        self.register_x = self.register_a;
        let value = self.get_operand(mode);
        self.register_a &= value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn axa(&mut self, mode: &AddressingMode) {
        let addr = mode.get_operand_address(self).unwrap();
        let value = self.register_a & self.register_x & (((addr >> 8) as u8) + 1);
        self.mem_write(addr, value);
    }

    fn xas(&mut self, mode: &AddressingMode) {
        self.register_s = self.register_a & self.register_x;
        let addr = mode.get_operand_address(self).unwrap();
        let value = self.register_s & (((addr >> 8) as u8) + 1);
        self.mem_write(addr, value);
    }

    fn sya(&mut self, mode: &AddressingMode) {
        let addr = mode.get_operand_address(self).unwrap();
        let value = self.register_y & (((addr >> 8) as u8) + 1);
        self.mem_write(addr, value);
    }

    fn sxa(&mut self, mode: &AddressingMode) {
        let addr = mode.get_operand_address(self).unwrap();
        let value = self.register_x & (((addr >> 8) as u8) + 1);
        self.mem_write(addr, value);
    }

    fn atx(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
        // some sources say that it should be register_a &= value
        self.register_a = value;
        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn lar(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode) & self.register_s;
        self.register_a = value;
        self.register_x = value;
        self.register_s = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn axs(&mut self, mode: &AddressingMode) {
        let value = self.get_operand(mode);
        self.register_x = (self.register_a & self.register_x).wrapping_sub(value);
        self.update_zero_and_negative_flags(self.register_x);
    }
}
