use itertools::Itertools;
use crate::bus::Mem;
use crate::cpu::{AddressingMode, CPU};
use crate::cpu::opcodes::{CPU_INSTRUCTIONS};

//TODO: cpu mut not good
pub fn trace(cpu: &mut CPU) -> String {
    let opcode = CPU_INSTRUCTIONS[cpu.mem_read(cpu.program_counter) as usize];
    let mut instruction_bytes = Vec::new();
    for i in 0..opcode.size {
        instruction_bytes.push(cpu.mem_read(cpu.program_counter + i));
    }
    let opcode_string = instruction_bytes.iter().map(|b| format!("{:02X}", *b)).join(" ");


    let mut asm = opcode.mnemonics.to_string();
    let operand_addr = opcode.mode.get_operand_address(cpu).unwrap_or(0);
    asm.push_str(&match opcode.mode {
        AddressingMode::Accumulator => " A".to_string(),
        AddressingMode::Immediate => format!(" #${:02X}", cpu.mem_read(operand_addr)),
        AddressingMode::Relative => {
            format!(" ${:02X}",operand_addr + opcode.size)
        }
        AddressingMode::ZeroPage => {
            let mut output = format!(" ${:02X}", operand_addr);
            let value = cpu.mem_read(operand_addr);
            output.push_str(&format!(" = {value:02X}"));
            output
        },
        AddressingMode::ZeroPage_X => {
            let value = cpu.mem_read(operand_addr);
            format!(" ${:02X},X @ {:02X} = {:02X}", instruction_bytes[1], operand_addr, value)
        },
        AddressingMode::ZeroPage_Y => {
            let value = cpu.mem_read(operand_addr);
            format!(" ${:02X},Y @ {:02X} = {:02X}", instruction_bytes[1], operand_addr, value)
        },
        AddressingMode::Absolute => {
            let mut output = format!(" ${:02X}{:02X}", instruction_bytes[2], instruction_bytes[1]);
            if opcode.mnemonics != "JMP" && opcode.mnemonics != "JSR" {
                let value = cpu.mem_read(operand_addr);
                output.push_str(&format!(" = {value:02X}"));
            }
            output
        },
        AddressingMode::Absolute_X => {
            let value = cpu.mem_read(operand_addr);
            format!(" ${:02X}{:02X},X @ {:04X} = {:02X} ", instruction_bytes[2], instruction_bytes[1], operand_addr, value)
        }
        AddressingMode::Absolute_Y => {
            let value = cpu.mem_read(operand_addr);
            format!(" ${:02X}{:02X},Y @ {:04X} = {:02X} ", instruction_bytes[2], instruction_bytes[1], operand_addr, value)
        }
        AddressingMode::Indirect => {
            let address = (instruction_bytes[2] as u16) << 8 | (instruction_bytes[1] as u16);
            let value = operand_addr;
            format!(" (${:04X}) = {:04X}", address, value)
        },
        AddressingMode::Indirect_X => {
            let address_location = cpu.register_x.wrapping_add(instruction_bytes[1]);
            let value = cpu.mem_read(operand_addr);
            format!(" (${:02X},X) @ {:02X} = {:04X} = {:02X}", instruction_bytes[1], address_location, operand_addr, value)
        },
        AddressingMode::Indirect_Y => {
            let lo = cpu.mem_read(instruction_bytes[1] as u16);
            let hi = cpu.mem_read(instruction_bytes[1].wrapping_add(1) as u16);
            let deref_base = (hi as u16) << 8 | (lo as u16);
            let value = cpu.mem_read(operand_addr);
            format!(" (${:02X}),Y = {:04X} @ {:04X} = {:02X}", instruction_bytes[1], deref_base, operand_addr, value)
        },
        AddressingMode::NonAddressing => { String::new() }
    });
    if opcode.mnemonics.len() == 3 {
        format!("{:04X}  {:<8}  {:<31} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}", cpu.program_counter, opcode_string, asm, cpu.register_a, cpu.register_x, cpu.register_y, cpu.status, cpu.register_s)
    } else {
        format!("{:04X}  {:<8} {:<32} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}", cpu.program_counter, opcode_string, asm, cpu.register_a, cpu.register_x, cpu.register_y, cpu.status, cpu.register_s)
    }
}