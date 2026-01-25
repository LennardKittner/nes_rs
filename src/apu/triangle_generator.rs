use crate::apu::length_counter::LengthCounter;
use crate::apu::timer::Timer;

pub struct TriangleGenerator {
    timer: Timer,
    length_counter: LengthCounter,
    counter_reload: bool,
    counter_reload_value: u8,
    control_flag: bool,
    counter: u8,
    current_sequencer_index: usize,
}

impl TriangleGenerator {
    const SEQUENCER_VALUES: [f32; 32] = [
        15f32, 14f32, 13f32, 12f32, 11f32, 10f32, 9f32, 8f32, 7f32, 6f32, 5f32, 4f32, 3f32, 2f32,
        1f32, 0f32, 0f32, 1f32, 2f32, 3f32, 4f32, 5f32, 6f32, 7f32, 8f32, 9f32, 10f32, 11f32,
        12f32, 13f32, 14f32, 15f32,
    ];

    pub fn new() -> Self {
        TriangleGenerator {
            timer: Timer::new(),
            length_counter: LengthCounter::new(),
            counter_reload: false,
            counter_reload_value: 0,
            control_flag: false,
            counter: 0,
            current_sequencer_index: 0,
        }
    }

    pub fn set_control_flag(&mut self, value: bool) {
        self.control_flag = value;
    }

    pub fn set_counter_reload_value(&mut self, value: u8) {
        self.counter_reload_value = value;
    }

    pub fn set_timer_lower(&mut self, timer: u8) {
        self.timer.data = (self.timer.data & 0xFF00) | timer as u16;
        self.timer.timer_limit = (self.timer.timer_limit & 0xFF00) | timer as u16;
    }

    pub fn set_timer_upper(&mut self, timer: u8) {
        self.timer.data = (self.timer.data & 0x00FF) | ((timer as u16) << 8);
        self.timer.timer_limit = (self.timer.timer_limit & 0x00FF) | ((timer as u16) << 8);
    }

    pub fn set_length_counter_value(&mut self, value: u8) {
        self.length_counter.set_length(value);
        self.counter_reload = true;
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
        if self.timer.tick(1) && self.counter != 0 && self.length_counter.get_value() != 0 {
            self.current_sequencer_index += 1;
            self.current_sequencer_index %= 32;
        }
    }

    pub fn tick_half_frame(&mut self) {
        self.tick_quarter_frame();
        self.length_counter.tick();
    }

    pub fn tick_quarter_frame(&mut self) {
        if self.counter_reload {
            self.counter = self.counter_reload_value;
        } else if self.counter > 0 {
            self.counter -= 1;
        }
        if !self.control_flag {
            self.counter_reload = false;
        }
    }

    pub fn get_output(&self) -> f32 {
        if self.counter == 0 || self.length_counter.get_value() == 0 {
            return 0f32;
        }
        if self.timer.timer_limit < 2 {
            return 0f32;
        }

        Self::SEQUENCER_VALUES[self.current_sequencer_index]
    }
}
