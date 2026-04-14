use crate::bus::Mem;
use crate::cpu::CPU;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Accumulator,
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

pub trait InstructionType {
    const STORE: bool = false;
}

pub struct Load;
pub struct Store;

impl InstructionType for Load {
    const STORE: bool = false;
}

impl InstructionType for Store {
    const STORE: bool = true;
}

pub fn page_cross(addr1: u16, addr2: u16) -> bool {
    (addr1 & 0xFF00) != (addr2 & 0xFF00)
}

//TODO: cycle accurate CPU by adding ticks here
impl AddressingMode {
    /// get the address of an operand
    /// returns none for AddressingMode::Accumulator | AddressingMode::NonAddressing
    /// otherwise returns (address, dummy_read, page_boundary_cross)
    pub fn get_operand_address<IT: InstructionType>(
        &self,
        cpu: &mut CPU,
    ) -> Option<(u16, bool, bool)> {
        let operand_location = cpu.program_counter.wrapping_add(1);
        match self {
            AddressingMode::Immediate => Some((operand_location, false, false)),
            AddressingMode::ZeroPage => Some((cpu.mem_read(operand_location) as u16, false, false)),
            AddressingMode::ZeroPage_X => {
                let pos = cpu.mem_read(operand_location);
                let _ = cpu.mem_read(pos as u16);
                Some((pos.wrapping_add(cpu.register_x) as u16, true, false))
            }
            AddressingMode::ZeroPage_Y => {
                let pos = cpu.mem_read(operand_location);
                let _ = cpu.mem_read(pos as u16);
                Some((pos.wrapping_add(cpu.register_y) as u16, true, false))
            }
            AddressingMode::Absolute => Some((cpu.mem_read_u16(operand_location), false, false)),
            AddressingMode::Absolute_X => {
                let base = cpu.mem_read_u16(operand_location);
                let target = base.wrapping_add(cpu.register_x as u16);
                let mut dummy_read_happened = false;
                let page_crossed = page_cross(base, target);
                if page_crossed && cpu.current_instruction.bonus_cycle_on_page_cross {
                    cpu.additional_cycles += 1;
                }
                if IT::STORE || page_crossed {
                    let dummy_read = (base & 0xFF00) | (target & 0x00FF);
                    let _ = cpu.mem_read(dummy_read);
                    dummy_read_happened = true;
                }
                Some((target, dummy_read_happened, page_crossed))
            }
            AddressingMode::Absolute_Y => {
                let base = cpu.mem_read_u16(operand_location);
                let target = base.wrapping_add(cpu.register_y as u16);
                let mut dummy_read_happened = false;
                let page_crossed = page_cross(base, target);
                if page_crossed && cpu.current_instruction.bonus_cycle_on_page_cross {
                    cpu.additional_cycles += 1;
                }
                if IT::STORE || page_crossed {
                    let dummy_read = (base & 0xFF00) | (target & 0x00FF);
                    let _ = cpu.mem_read(dummy_read);
                    dummy_read_happened = true;
                }
                Some((target, dummy_read_happened, page_crossed))
            }
            // An original 6502 does not correctly fetch the target address if the indirect vector falls on a page boundary (e.g. $xxFF where xx is any value from $00 to $FF).
            // In this case fetches the LSB from $xxFF as expected but takes the MSB from $xx00.
            AddressingMode::Indirect => {
                let address = cpu.mem_read_u16(operand_location);
                if address & 0xFF == 0xFF {
                    let lo = cpu.mem_read(address);
                    let hi = cpu.mem_read(address & 0xFF00);
                    Some(((hi as u16) << 8 | (lo as u16), false, false))
                } else {
                    Some((cpu.mem_read_u16(address), false, false))
                }
            }
            AddressingMode::Indirect_X => {
                let base = cpu.mem_read(operand_location);
                let ptr = base.wrapping_add(cpu.register_x);
                let lo = cpu.mem_read(ptr as u16);
                let hi = cpu.mem_read(ptr.wrapping_add(1) as u16);
                Some(((hi as u16) << 8 | (lo as u16), false, false))
            }
            AddressingMode::Indirect_Y => {
                let base = cpu.mem_read(operand_location);
                let lo = cpu.mem_read(base as u16);
                let hi = cpu.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let target = deref_base.wrapping_add(cpu.register_y as u16);
                let mut dummy_read_happened = false;
                let page_crossed = page_cross(deref_base, target);
                if page_crossed && cpu.current_instruction.bonus_cycle_on_page_cross {
                    cpu.additional_cycles += 1;
                }
                if IT::STORE || page_crossed {
                    let dummy_read = (deref_base & 0xFF00) | (target & 0x00FF);
                    let _ = cpu.mem_read(dummy_read);
                    dummy_read_happened = true;
                }
                Some((target, dummy_read_happened, page_crossed))
            }
            AddressingMode::Relative => {
                let mut offset = cpu.mem_read(operand_location);
                if offset & 0b1000_0000 != 0 {
                    offset = offset.wrapping_neg();
                    Some((
                        cpu.program_counter.wrapping_sub(offset as u16),
                        false,
                        false,
                    ))
                } else {
                    Some((
                        cpu.program_counter.wrapping_add(offset as u16),
                        false,
                        false,
                    ))
                }
            }
            AddressingMode::Accumulator | AddressingMode::NonAddressing => None,
        }
    }

    pub fn trace_get_operand_address(&self, cpu: &CPU) -> Option<u16> {
        let operand_location = cpu.program_counter.wrapping_add(1);
        match self {
            AddressingMode::Immediate => Some(operand_location),
            AddressingMode::ZeroPage => Some(cpu.trace_mem_read(operand_location) as u16),
            AddressingMode::ZeroPage_X => {
                let pos = cpu.trace_mem_read(operand_location);
                Some(pos.wrapping_add(cpu.register_x) as u16)
            }
            AddressingMode::ZeroPage_Y => {
                let pos = cpu.trace_mem_read(operand_location);
                Some(pos.wrapping_add(cpu.register_y) as u16)
            }
            AddressingMode::Absolute => Some(cpu.trace_mem_read_u16(operand_location)),
            AddressingMode::Absolute_X => {
                let base = cpu.trace_mem_read_u16(operand_location);
                Some(base.wrapping_add(cpu.register_x as u16))
            }
            AddressingMode::Absolute_Y => {
                let base = cpu.trace_mem_read_u16(operand_location);
                Some(base.wrapping_add(cpu.register_y as u16))
            }
            // An original 6502 does not correctly fetch the target address if the indirect vector falls on a page boundary (e.g. $xxFF where xx is any value from $00 to $FF).
            // In this case fetches the LSB from $xxFF as expected but takes the MSB from $xx00.
            AddressingMode::Indirect => {
                let address = cpu.trace_mem_read_u16(operand_location);
                if address & 0xFF == 0xFF {
                    let lo = cpu.trace_mem_read(address);
                    let hi = cpu.trace_mem_read(address & 0xFF00);
                    Some((hi as u16) << 8 | (lo as u16))
                } else {
                    Some(cpu.trace_mem_read_u16(address))
                }
            }
            AddressingMode::Indirect_X => {
                let base = cpu.trace_mem_read(operand_location);
                let ptr = base.wrapping_add(cpu.register_x);
                let lo = cpu.trace_mem_read(ptr as u16);
                let hi = cpu.trace_mem_read(ptr.wrapping_add(1) as u16);
                Some((hi as u16) << 8 | (lo as u16))
            }
            AddressingMode::Indirect_Y => {
                let base = cpu.trace_mem_read(operand_location);
                let lo = cpu.trace_mem_read(base as u16);
                let hi = cpu.trace_mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                Some(deref_base.wrapping_add(cpu.register_y as u16))
            }
            AddressingMode::Relative => {
                let mut offset = cpu.trace_mem_read(operand_location);
                if offset & 0b1000_0000 != 0 {
                    offset = offset.wrapping_neg();
                    Some(cpu.program_counter.wrapping_sub(offset as u16))
                } else {
                    Some(cpu.program_counter + (offset as u16))
                }
            }
            AddressingMode::Accumulator | AddressingMode::NonAddressing => None,
        }
    }
}
