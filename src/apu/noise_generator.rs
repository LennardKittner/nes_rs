use crate::apu::envelope::EnvelopeGenerator;
use crate::apu::length_counter::LengthCounter;
use crate::apu::timer::Timer;

pub struct NoiseGenerator {
    timer: Timer,
    length_counter: LengthCounter,
    envelope_generator: EnvelopeGenerator,
    loop_mode: bool,
    shift_register: u16,
}

impl NoiseGenerator {
    //NTSC
    const PERIOD_TABLE: [u16; 16] = [
        4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
    ];
    pub fn new() -> Self {
        NoiseGenerator {
            timer: Timer::new(),
            length_counter: LengthCounter::new(),
            envelope_generator: EnvelopeGenerator::new(),
            loop_mode: false,
            shift_register: 1,
        }
    }

    pub fn set_period(&mut self, period: u8) {
        let limit = Self::PERIOD_TABLE[period as usize] / 2; // noise is only ticked every other cpu cycle
        self.timer.timer_limit = limit;
        self.timer.data = limit;
    }

    pub fn set_loop_mode(&mut self, loop_mode: bool) {
        self.loop_mode = loop_mode;
    }

    pub fn set_length_counter_value(&mut self, value: u8) {
        self.length_counter.set_length(value);
        self.envelope_generator.set_start(true);
    }

    pub fn set_length_counter_halt(&mut self, halt: bool) {
        self.length_counter.set_halt(halt);
    }

    pub fn set_length_counter_enabled(&mut self, enabled: bool) {
        self.length_counter.set_enabled(enabled);
    }

    pub fn is_active(&self) -> bool {
        self.length_counter.get_value() > 0
    }

    pub fn tick(&mut self) {
        if self.timer.tick(1) {
            let feed_back = (self.shift_register & 1)
                ^ if self.loop_mode {
                    (self.shift_register & 0b0000_0000_0100_0000) >> 6
                } else {
                    (self.shift_register & 0b0000_0000_0000_0010) >> 1
                };
            self.shift_register >>= 1;
            self.shift_register |= feed_back << 14;
        }
    }

    pub fn tick_half_frame(&mut self) {
        self.tick_quarter_frame();
        self.length_counter.tick();
    }

    pub fn tick_quarter_frame(&mut self) {
        self.envelope_generator.tick();
    }

    pub fn set_envelope_parameters(
        &mut self,
        envelope_loop: bool,
        constant_volume: bool,
        volume_envelope: u8,
    ) {
        self.envelope_generator.set_loop_envelope(envelope_loop);
        self.envelope_generator.set_constant_volume(constant_volume);
        self.envelope_generator.set_start(true);
        if constant_volume {
            self.envelope_generator.set_volume(volume_envelope);
        } else {
            self.envelope_generator.set_envelope(volume_envelope)
        }
    }

    pub fn get_output(&self) -> f32 {
        if self.shift_register & 1 == 1 || self.length_counter.get_value() == 0  {
            return 0f32;
        }

        self.envelope_generator.get_volume_normalized()
    }
}
