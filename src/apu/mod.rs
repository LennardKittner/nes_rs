use crate::apu::pulse_generator::{PulseGenerator, PulseGeneratorID};
use crate::apu::ring_buffer::RingBuffer;
use crate::bus::PollIRQ;

mod envelope;
mod pulse_generator;
mod ring_buffer;
mod sweep_unit;
mod timer;

#[derive(Eq, PartialEq)]
enum FrameCounterMode {
    MODE5STEP,
    MODE4STEP,
}

pub struct APU {
    pulse_generator1: PulseGenerator,
    pulse_generator2: PulseGenerator,
    ring_buffer: RingBuffer<f32, 44100>, // 1s of audio
    cycle_in_frame: usize,
    enable_interrupt: bool,
    frame_counter_mode: FrameCounterMode,
    outstanding_interrupt: bool,
}

impl PollIRQ for APU {
    fn poll_irq(&mut self) -> bool {
        if self.outstanding_interrupt {
            self.outstanding_interrupt = false;
            true
        } else {
            false
        }
    }
}

impl APU {
    pub fn new() -> APU {
        Self {
            pulse_generator1: PulseGenerator::new(PulseGeneratorID::One),
            pulse_generator2: PulseGenerator::new(PulseGeneratorID::Two),
            ring_buffer: RingBuffer::new(),
            cycle_in_frame: 0,
            enable_interrupt: false,
            frame_counter_mode: FrameCounterMode::MODE4STEP,
            outstanding_interrupt: false,
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.cycle_in_frame += 1;
            if self.cycle_in_frame % 2 == 1 {
                continue;
            }
            if self.frame_counter_mode == FrameCounterMode::MODE5STEP {
                self.tick_5_step()
            } else {
                self.tick_4_step()
            }

            self.pulse_generator1.tick();
            self.pulse_generator2.tick();
            self.ring_buffer.push(self.get_output());
        }
    }

    fn tick_4_step(&mut self) {
        match self.cycle_in_frame {
            c if c >= 14914 => {
                self.pulse_generator1.tick_half_frame();
                self.pulse_generator2.tick_half_frame();
                if self.enable_interrupt {
                    self.outstanding_interrupt = true;
                }
                self.cycle_in_frame = 0;
            }
            c if c >= 11185 => {
                self.pulse_generator1.tick_quarter_frame();
                self.pulse_generator2.tick_quarter_frame();
            }
            c if c >= 7456 => {
                self.pulse_generator1.tick_half_frame();
                self.pulse_generator2.tick_half_frame();
            }
            c if c >= 3728 => {
                self.pulse_generator1.tick_quarter_frame();
                self.pulse_generator2.tick_quarter_frame();
            }
            _ => {}
        }
    }

    fn tick_5_step(&mut self) {
        match self.cycle_in_frame {
            c if c >= 18640 => {
                self.pulse_generator1.tick_half_frame();
                self.pulse_generator2.tick_half_frame();
                self.cycle_in_frame = 0;
            }
            c if c >= 14914 => { /* Do Nothing */ }
            c if c >= 11185 => {
                self.pulse_generator1.tick_quarter_frame();
                self.pulse_generator2.tick_quarter_frame();
            }
            c if c >= 7456 => {
                self.pulse_generator1.tick_half_frame();
                self.pulse_generator2.tick_half_frame();
            }
            c if c >= 3728 => {
                self.pulse_generator1.tick_quarter_frame();
                self.pulse_generator2.tick_quarter_frame();
            }
            _ => {}
        }
    }

    pub fn get_output(&self) -> f32 {
        let pulse1 = self.pulse_generator1.get_output();
        let pulse2 = self.pulse_generator2.get_output();
        if pulse1 + pulse2 <= 0f32 {
            0f32
        } else {
            95.88f32 / ((8128f32 / (pulse1 + pulse2)) + 100f32)
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        self.ring_buffer.next().unwrap_or(0f32)
    }

    /// ---D NT21
    /// Enable DMC (D), noise (N), triangle (T), and pulse channels (2/1)
    pub fn get_status(&self) -> u8 {
        let mut status = 0;
        if self.pulse_generator1.is_active() {
            status |= 0b0000_0001;
        }
        if self.pulse_generator2.is_active() {
            status |= 0b0000_0010;
        }
        //todo!()
        status
    }

    /// IF-D NT21
    /// DMC interrupt (I), frame interrupt (F), DMC active (D), length counter > 0 (N/T/2/1)
    pub fn set_status(&mut self, value: u8) {
        self.pulse_generator1
            .set_length_counter_enabled(value & 0b0000_0001 != 0);
        self.pulse_generator2
            .set_length_counter_enabled(value & 0b0000_0010 != 0);
        //todo!()
    }

    /// MI-- ----
    /// Mode (M, 0 = 4-step, 1 = 5-step), IRQ inhibit flag (I)
    pub fn set_frame_counter(&mut self, value: u8) {
        if value & 0b1000_0000 != 0 {
            self.frame_counter_mode = FrameCounterMode::MODE4STEP;
        } else {
            self.frame_counter_mode = FrameCounterMode::MODE5STEP;
        }
        self.enable_interrupt = value & 0b0100_0000 == 0;
    }

    /// DDLC VVVV
    /// Duty (D), envelope loop / length counter halt (L), constant volume (C), volume/envelope (V)
    pub fn set_pulse1_DLCV(&mut self, value: u8) {
        self.pulse_generator1.set_duty((value & 0b1100_0000) >> 6);
        let l = value & 0b0010_0000 != 0;
        self.pulse_generator1.set_length_counter_halt(l);
        self.pulse_generator1
            .set_envelope_parameters(l, value & 0b0001_0000 != 0, value & 0x0F);
    }

    /// DDLC VVVV
    /// Duty (D), envelope loop / length counter halt (L), constant volume (C), volume/envelope (V)
    pub fn set_pulse2_DLCV(&mut self, value: u8) {
        self.pulse_generator2.set_duty((value & 0b1100_0000) >> 6);
        let l = value & 0b0010_0000 != 0;
        self.pulse_generator2.set_length_counter_halt(l);
        self.pulse_generator2
            .set_envelope_parameters(l, value & 0b0001_0000 != 0, value & 0x0F);
    }

    /// EPPP NSSS
    /// Sweep unit: enabled (E), period (P), negate (N), shift (S)
    pub fn set_pulse1_EPNS(&mut self, value: u8) {
        self.pulse_generator1.set_sweep_parameters(
            value & 0b1000_0000 != 0,
            value & 0b0000_1000 != 0,
            value & 0b0000_0111,
            (value & 0b0111_0000) >> 4,
        )
    }

    /// EPPP NSSS
    /// Sweep unit: enabled (E), period (P), negate (N), shift (S)
    pub fn set_pulse2_EPNS(&mut self, value: u8) {
        self.pulse_generator2.set_sweep_parameters(
            value & 0b1000_0000 != 0,
            value & 0b0000_1000 != 0,
            value & 0b0000_0111,
            (value & 0b0111_0000) >> 4,
        )
    }

    /// TTTT TTTT
    /// Timer low (T)
    pub fn set_pulse1_timer_low(&mut self, value: u8) {
        self.pulse_generator1.set_timer_lower(value)
    }

    /// TTTT TTTT
    /// Timer low (T)
    pub fn set_pulse2_timer_low(&mut self, value: u8) {
        self.pulse_generator2.set_timer_lower(value)
    }

    /// LLLL LTTT
    /// Length counter load (L), timer high (T)
    pub fn set_pulse1_LT(&mut self, value: u8) {
        self.pulse_generator1.set_length_counter_value(value >> 3);
        self.pulse_generator1.set_timer_upper(value & 0x7)
    }

    /// LLLL LTTT
    /// Length counter load (L), timer high (T)
    pub fn set_pulse2_LT(&mut self, value: u8) {
        self.pulse_generator2.set_length_counter_value(value >> 3);
        self.pulse_generator2.set_timer_upper(value & 0x7)
    }

    /// CRRR RRRR
    /// Length counter halt / linear counter control (C), linear counter load (R)
    pub fn set_triangle_CR(&mut self, value: u8) {
        //todo!()
    }

    /// TTTT TTTT
    /// Timer low (T)
    pub fn set_triangle_timer_low(&mut self, value: u8) {
        //todo!()
    }

    /// LLLL LTTT
    /// Length counter load (L), timer high (T), set linear counter reload flag
    pub fn set_triangle_LT(&mut self, value: u8) {
        //todo!()
    }

    /// -LC VVVV
    /// Envelope loop / length counter halt (L), constant volume (C), volume/envelope (V)
    pub fn set_noise_LCV(&mut self, value: u8) {
        //todo!()
    }

    /// L--- PPPP
    /// Loop noise (L), noise period (P)
    pub fn set_noise_LP(&mut self, value: u8) {
        //todo!()
    }

    /// LLLL L---
    /// Length counter load (L)
    pub fn set_noise_length_counter_load(&mut self, value: u8) {
        //todo!()
    }

    /// IL-- RRRR
    /// IRQ enable (I), loop (L), frequency (R)
    pub fn set_DMC_ILR(&mut self, value: u8) {
        //todo!()
    }

    /// -DDD DDDD
    /// Load counter (D)
    pub fn set_DMC_load_counter(&mut self, value: u8) {
        //todo!()
    }

    /// AAAA AAAA
    /// Sample address (A)
    pub fn set_DMC_sample_address(&mut self, value: u8) {
        //todo!()
    }

    /// LLLL LLLL
    /// Sample length (L)
    pub fn set_DMC_sample_length(&mut self, value: u8) {
        //todo!()
    }
}
