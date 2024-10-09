use crate::apu::timer::Timer;
use crate::bus::{Bus, Mem};
use std::ops::Sub;

struct MemoryReader {
    pub sample_start_address: u16,
    pub sample_address: u16,
    pub sample_length: u16,
    pub bytes_remaining: u16,
    pub sample_buffer: Option<u8>,
}

impl MemoryReader {
    fn new() -> Self {
        MemoryReader {
            sample_start_address: 0,
            sample_address: 0,
            sample_length: 0,
            bytes_remaining: 0,
            sample_buffer: None,
        }
    }

    fn load_new_sample(&mut self, bus: &mut Bus, loop_flag: bool) -> bool {
        //TODO: The CPU is stalled for 1-4 CPU cycles to read a sample byte. The exact cycle count depends on many factors and is described in detail in the DMA article.
        self.sample_buffer = Some(bus.mem_read(self.sample_address));
        if self.sample_address == 0xFFFF {
            self.sample_address = 0x8000;
        } else {
            self.sample_address += 1;
        }
        self.bytes_remaining -= 1;
        if self.bytes_remaining == 0 {
            if loop_flag {
                self.restart();
            } else {
                return true;
            }
        }
        false
    }

    pub fn restart(&mut self) {
        self.bytes_remaining = self.sample_length;
        self.sample_address = self.sample_start_address;
    }
}

struct OutputUnit {
    shift_register: u8,
    bit_remaining: u8,
    output_buffer: u8,
    silence: bool,
}

impl OutputUnit {
    fn new() -> Self {
        OutputUnit {
            shift_register: 0,
            bit_remaining: 8,
            output_buffer: 0,
            silence: false,
        }
    }

    fn tick(&mut self, memory_reader: &mut MemoryReader) {
        if !self.silence {
            if self.shift_register & 1 == 1 {
                if self.output_buffer <= 125 {
                    self.output_buffer += 2;
                }
            } else if self.output_buffer >= 2 {
                self.output_buffer = self.output_buffer.sub(2);
            }
            self.shift_register >>= 1;
        }

        self.bit_remaining -= 1;
        if self.bit_remaining == 0 {
            self.bit_remaining = 8;
            match memory_reader.sample_buffer.take() {
                Some(buffer) => {
                    self.shift_register = buffer;
                    self.silence = false;
                }
                None => self.silence = true,
            }
        }
    }
}

pub struct DataModulationChannel {
    timer: Timer,
    memory_reader: MemoryReader,
    output_unit: OutputUnit,
    irq_enabled: bool,
    loop_flag: bool,
    outstanding_interrupt: bool,
}

impl DataModulationChannel {
    // NTSC
    pub const OUTPUT_RATE_TABLE: [u16; 16] = [
        428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
    ];
    pub fn new() -> Self {
        DataModulationChannel {
            timer: Timer::new(),
            memory_reader: MemoryReader::new(),
            output_unit: OutputUnit::new(),
            irq_enabled: false,
            loop_flag: false,
            outstanding_interrupt: false,
        }
    }

    pub fn poll_irq(&mut self) -> bool {
        if self.outstanding_interrupt {
            self.outstanding_interrupt = false;
            true
        } else {
            false
        }
    }

    pub fn set_irq_enable(&mut self, enabled: bool) {
        self.irq_enabled = enabled;
    }

    pub fn set_loop_flag(&mut self, enabled: bool) {
        self.loop_flag = enabled;
    }

    pub fn set_output_rate(&mut self, value: u8) {
        self.timer.timer_limit = Self::OUTPUT_RATE_TABLE[value as usize] / 2; // only clocked every second CPU cycle;
        self.timer.data = Self::OUTPUT_RATE_TABLE[value as usize] / 2; // only clocked every second CPU cycle
    }

    pub fn direct_load(&mut self, value: u8) {
        self.output_unit.output_buffer = value;
    }

    pub fn set_sample_address(&mut self, value: u8) {
        self.memory_reader.sample_start_address = 0xC000 + value as u16 * 64;
        self.memory_reader.sample_address = self.memory_reader.sample_start_address;
    }

    pub fn set_sample_length(&mut self, value: u8) {
        self.memory_reader.sample_length = 1 + value as u16 * 16;
    }

    pub fn set_bytes_remaining(&mut self, value: u16) {
        self.memory_reader.bytes_remaining = value;
    }

    pub fn restart(&mut self) {
        self.memory_reader.restart();
    }

    pub fn tick(&mut self, bus: &mut Bus) {
        if self.timer.tick(1) {
            if self.memory_reader.bytes_remaining != 0 && self.memory_reader.sample_buffer.is_none()
            {
                self.outstanding_interrupt |=
                    self.irq_enabled & self.memory_reader.load_new_sample(bus, self.loop_flag);
            }
            self.output_unit.tick(&mut self.memory_reader);
        }
    }

    pub fn is_interrupt_enabled(&self) -> bool {
        self.irq_enabled
    }

    pub fn get_bytes_remaining(&self) -> u16 {
        self.memory_reader.bytes_remaining
    }

    pub fn get_output(&self) -> f32 {
        self.output_unit.output_buffer as f32
    }
}
