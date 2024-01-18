use itertools::Itertools;
use crate::bus::Mem;
use crate::cpu::{AddressingMode, CPU};
use crate::cpu::opcodes::CPU_INSTRUCTIONS;

pub fn trace(cpu: &CPU) -> String {
    let opcode = &CPU_INSTRUCTIONS[&cpu.mem_read(cpu.program_counter)];
    let mut instruction_bytes = Vec::new();
    for i in 0..opcode.size {
        instruction_bytes.push(cpu.mem_read(cpu.program_counter + i));
    }
    let opcode_string = instruction_bytes.iter().map(|b| format!("{:02X}", *b)).join(" ");


    let mut asm = opcode.name.to_string();
    asm.push_str(&match opcode.mode {
        AddressingMode::Accumulator => " A".to_string(),
        AddressingMode::Immediate => format!(" #${:02X}", instruction_bytes[1]),
        AddressingMode::Relative => {
            format!(" ${:02X}", cpu.program_counter.wrapping_add(instruction_bytes[1] as u16).wrapping_add(opcode.size))
        }
        AddressingMode::ZeroPage => {
            let mut output = format!(" ${:02X}", instruction_bytes[1]);
            let value = cpu.mem_read(instruction_bytes[1] as u16);
            output.push_str(&format!(" = {value:02X}"));
            output
        },
        AddressingMode::ZeroPage_X => {
            let address = instruction_bytes[1].wrapping_add(cpu.register_x) as u16;
            let value = cpu.mem_read(address);
            format!(" ${:02X},X @ {:02X} = {:02X}", instruction_bytes[1], address, value)
        },
        AddressingMode::ZeroPage_Y => {
            let address = instruction_bytes[1].wrapping_add(cpu.register_y) as u16;
            let value = cpu.mem_read(address);
            format!(" ${:02X},Y @ {:02X} = {:02X}", instruction_bytes[1], address, value)
        },
        AddressingMode::Absolute => {
            let mut output = format!(" ${:02X}{:02X}", instruction_bytes[2], instruction_bytes[1]);
            if opcode.name != "JMP" && opcode.name != "JSR" {
                let value = cpu.mem_read(((instruction_bytes[2] as u16) << 8) | instruction_bytes[1] as u16);
                output.push_str(&format!(" = {value:02X}"));
            }
            output
        },
        AddressingMode::Absolute_X => {
            let address = (((instruction_bytes[2] as u16) << 8) | instruction_bytes[1] as u16).wrapping_add(cpu.register_x as u16);
            let value = cpu.mem_read(address);
            format!(" ${:02X}{:02X},X @ {:04X} = {:02X} ", instruction_bytes[2], instruction_bytes[1], address, value)
        }
        AddressingMode::Absolute_Y => {
            let address = (((instruction_bytes[2] as u16) << 8) | instruction_bytes[1] as u16).wrapping_add(cpu.register_y as u16);
            let value = cpu.mem_read(address);
            format!(" ${:02X}{:02X},Y @ {:04X} = {:02X} ", instruction_bytes[2], instruction_bytes[1], address, value)
        }
        AddressingMode::Indirect => {
            let address = (instruction_bytes[2] as u16) << 8 | (instruction_bytes[1] as u16);
            let value = if address & 0xFF == 0xFF {
                let lo = cpu.mem_read(address);
                let hi = cpu.mem_read(address & 0xFF00);
                (hi as u16) << 8 | (lo as u16)
            } else {
                cpu.mem_read_u16(address)
            };
            format!(" (${:04X}) = {:04X}", address, value)
        },
        AddressingMode::Indirect_X => {
            let address_location = cpu.register_x.wrapping_add(instruction_bytes[1]);
            let lo = cpu.mem_read(address_location as u16);
            let hi = cpu.mem_read(address_location.wrapping_add(1) as u16);
            let address = (hi as u16) << 8 | (lo as u16);
            let value = cpu.mem_read(address);
            format!(" (${:02X},X) @ {:02X} = {:04X} = {:02X}", instruction_bytes[1], address_location, address, value)
        },
        AddressingMode::Indirect_Y => {
            let lo = cpu.mem_read(instruction_bytes[1] as u16);
            let hi = cpu.mem_read(instruction_bytes[1].wrapping_add(1) as u16);
            let deref_base = (hi as u16) << 8 | (lo as u16);
            let address = deref_base.wrapping_add(cpu.register_y as u16);
            let value = cpu.mem_read(address);
            format!(" (${:02X}),Y = {:04X} @ {:04X} = {:02X}", instruction_bytes[1], deref_base, address, value)
        },
        AddressingMode::NonAddressing => { String::new() }
    });
    format!("{:04X}  {:<8}  {:<31} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}", cpu.program_counter, opcode_string, asm, cpu.register_a, cpu.register_x, cpu.register_y, cpu.status, cpu.register_s)
}