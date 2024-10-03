use crate::apu::pulse_generator::{PulseGenerator, PulseGeneratorID};
use crate::apu::ring_buffer::RingBuffer;

mod envelope;
mod pulse_generator;
mod ring_buffer;
mod sweep_unit;
mod timer;

pub struct APU {
    pulse_generator1: PulseGenerator,
    pulse_generator2: PulseGenerator,
    ring_buffer: RingBuffer<f32, 44100>, // 1s of audio
}

impl APU {
    pub fn new() -> APU {
        Self {
            pulse_generator1: PulseGenerator::new(PulseGeneratorID::ONE),
            pulse_generator2: PulseGenerator::new(PulseGeneratorID::TWO),
            ring_buffer: RingBuffer::new(),
        }
    }

    pub fn tick(&mut self, cycles: u8) {
        self.pulse_generator1.tick(cycles);
        self.pulse_generator2.tick(cycles);
        self.ring_buffer.push(self.get_output());
    }

    pub fn get_output(&self) -> f32 {
        self.pulse_generator1.get_output() + self.pulse_generator2.get_output()
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
        //todo!()
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
