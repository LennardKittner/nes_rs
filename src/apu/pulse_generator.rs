use crate::apu::envelope::EnvelopeGenerator;
use crate::apu::sweep_unit::SweepUnit;
use crate::apu::timer::Timer;

#[derive(Debug, Eq, Copy, Clone, PartialEq)]
pub enum PulseGeneratorID {
    ONE,
    TWO
}

pub struct PulseGenerator {
    timer: Timer,
    sweep_unit: SweepUnit,
    pulse_generator_id: PulseGeneratorID,
    envelope_generator: EnvelopeGenerator,
    duty: u8,
    duty_position: usize,
    length_counter_value: u8,
    length_counter_halt: bool,
    length_counter_enabled: bool,
}

impl PulseGenerator {
    const LENGTH_COUNTER_TABLE: [u8; 32] = [
        10, 254, 20,  2, 40,  4, 80,  6,
        160, 8, 60, 10, 14, 12, 26, 14,
        12, 16, 24, 18, 48, 20, 96, 22,
        192, 24, 72, 26, 16, 28, 32, 30
    ];

    const DUTY_PATTERNS: [[u8; 8]; 4] = [
        [0, 1, 0, 0, 0, 0, 0, 0], // 12.5% duty
        [0, 1, 1, 0, 0, 0, 0, 0], // 25% duty
        [0, 1, 1, 1, 1, 0, 0, 0], // 50% duty
        [1, 0, 0, 1, 1, 1, 1, 1], // 75% duty
    ];

    pub fn new(pulse_generator_id: PulseGeneratorID) -> PulseGenerator {
        PulseGenerator {
            timer: Timer::new(),
            sweep_unit: SweepUnit::new(pulse_generator_id),
            envelope_generator: EnvelopeGenerator::new(),
            pulse_generator_id,
            duty: 0,
            duty_position: 0,
            length_counter_value: 0,
            length_counter_halt: false,
            length_counter_enabled: false,
        }
    }

    pub fn set_duty(&mut self, value: u8) {
        self.duty = value;
    }

    pub fn set_length_counter_value(&mut self, value: u8) {
        self.length_counter_value = Self::LENGTH_COUNTER_TABLE[value as usize];
        self.envelope_generator.set_start(true);
    }

    pub fn set_length_counter_halt(&mut self, halt: bool) {
        self.length_counter_halt = halt;
    }

    pub fn set_length_counter_enabled(&mut self, enabled: bool) {
        self.length_counter_enabled = enabled;
        if !enabled {
            self.length_counter_value = 0;
        }
    }

    pub fn set_timer_lower(&mut self, timer: u8) {
        self.timer.data = (self.timer.data & 0xFF00) | timer as u16;
        self.timer.timer_limit = (self.timer.timer_limit & 0xFF00) | timer as u16;
    }

    pub fn set_timer_upper(&mut self, timer: u8) {
        self.timer.data = (self.timer.data & 0x00FF) | ((timer as u16) << 8);
        self.timer.timer_limit = (self.timer.timer_limit & 0x00FF) | ((timer as u16) << 8);
    }

    pub fn is_active(&self) -> bool {
        self.length_counter_value > 0
    }

    pub fn tick(&mut self, cycles: u8) {
        for i in 0..cycles {
            if self.timer.tick(1) {
                self.duty_position = (self.duty_position + 1) % 8;
            }
            if self.length_counter_value > 0 && !self.length_counter_halt && self.length_counter_enabled {
                self.length_counter_value -= 1;
            }
            self.timer.data = self.sweep_unit.tick(self.timer.data);
            self.envelope_generator.tick();
        }
    }

    pub fn get_output(&self) -> f32 {
        let patter = Self::DUTY_PATTERNS[self.duty as usize];

        //TODO: what about the rest
        if self.sweep_unit.should_mute(self.timer.data) || self.length_counter_value == 0 {
            return 0.0;
        }

        (if patter[self.duty_position] == 1 { 1f32 } else { 0f32 } * self.envelope_generator.get_volume_normalized())
    }

    pub fn set_sweep_parameters(&mut self, enabled: bool, negate: bool, shift: u8, period: u8) {
        self.sweep_unit.set_enable(enabled);
        self.sweep_unit.set_negate(negate);
        self.sweep_unit.set_shift(shift);
        self.sweep_unit.set_divider_period(period);
    }

    pub fn set_envelope_parameters(&mut self, envelope_loop: bool, constant_volume: bool, volume_envelope: u8) {
        self.envelope_generator.set_loop_envelope(envelope_loop);
        self.envelope_generator.set_constant_volume(constant_volume);
        self.envelope_generator.set_start(true);
        if constant_volume {
            self.envelope_generator.set_volume(volume_envelope);
        } else {
            self.envelope_generator.set_envelope(volume_envelope)
        }
    }
}