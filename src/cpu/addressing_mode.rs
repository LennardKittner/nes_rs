use crate::bus::Mem;
use crate::cpu::CPU;

//TODO: maybe inheritance instead of enums
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

pub fn page_cross(addr1: u16, addr2: u16) -> bool {
    (addr1 & 0xFF00) != (addr2 & 0xFF00)
}

impl AddressingMode {
    pub fn access_crosses_page_border(&self, cpu: &mut CPU) -> bool {
        let operand_location = cpu.program_counter + 1;
        match self {
            AddressingMode::Absolute_X => {
                let base = cpu.mem_read_u16(operand_location);
                let target = base.wrapping_add(cpu.register_x as u16);
                page_cross(base, target)
            }
            AddressingMode::Absolute_Y => {
                let base = cpu.mem_read_u16(operand_location);
                let target = base.wrapping_add(cpu.register_y as u16);
                page_cross(base, target)
            }
            AddressingMode::Indirect_Y => {
                let base = cpu.mem_read(operand_location);
                let lo = cpu.mem_read(base as u16);
                let hi = cpu.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref =  deref_base.wrapping_add(cpu.register_y as u16);
                page_cross(deref_base, deref)
            }
            _ => panic!("")
        }
    }

    pub fn get_operand_address(&self, cpu: &mut CPU) -> Option<u16> {
        let operand_location = cpu.program_counter.wrapping_add(1);
        match self {
            AddressingMode::Immediate => Some(operand_location),
            AddressingMode::ZeroPage => Some(cpu.mem_read(operand_location) as u16),
            AddressingMode::ZeroPage_X => {
                let pos = cpu.mem_read(operand_location);
                Some(pos.wrapping_add(cpu.register_x) as u16)
            }
            AddressingMode::ZeroPage_Y => {
                let pos = cpu.mem_read(operand_location);
                Some(pos.wrapping_add(cpu.register_y) as u16)
            }
            AddressingMode::Absolute => Some(cpu.mem_read_u16(operand_location)),
            AddressingMode::Absolute_X => {
                let base = cpu.mem_read_u16(operand_location);
                Some(base.wrapping_add(cpu.register_x as u16))
            }
            AddressingMode::Absolute_Y => {
                let base = cpu.mem_read_u16(operand_location);
                Some(base.wrapping_add(cpu.register_y as u16))
            }
            // An original 6502 does not correctly fetch the target address if the indirect vector falls on a page boundary (e.g. $xxFF where xx is any value from $00 to $FF).
            // In this case fetches the LSB from $xxFF as expected but takes the MSB from $xx00.
            AddressingMode::Indirect => {
                let address = cpu.mem_read_u16(operand_location);
                if address & 0xFF == 0xFF {
                    let lo = cpu.mem_read(address);
                    let hi = cpu.mem_read(address & 0xFF00);
                    Some((hi as u16) << 8 | (lo as u16))
                } else {
                    Some(cpu.mem_read_u16(address))
                }
            },
            AddressingMode::Indirect_X => {
                let base = cpu.mem_read(operand_location);
                let ptr = base.wrapping_add(cpu.register_x);
                let lo = cpu.mem_read(ptr as u16);
                let hi = cpu.mem_read(ptr.wrapping_add(1) as u16);
                Some((hi as u16) << 8 | (lo as u16))
            }
            AddressingMode::Indirect_Y => {
                let base = cpu.mem_read(operand_location);
                let lo = cpu.mem_read(base as u16);
                let hi = cpu.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                Some(deref_base.wrapping_add(cpu.register_y as u16))
            }
            AddressingMode::Relative => {
                let mut offset = cpu.mem_read(operand_location);
                if offset & 0b1000_0000 != 0 {
                    offset = offset.wrapping_neg();
                    Some(cpu.program_counter.wrapping_sub(offset as u16))
                } else {
                    Some(cpu.program_counter + (offset as u16))
                }
            }
            AddressingMode::Accumulator |
            AddressingMode::NonAddressing => None,
        }
    }
}
