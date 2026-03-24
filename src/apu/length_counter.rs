use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LengthCounter {
    value: u8,
    halt: bool,
    enabled: bool,
}

impl LengthCounter {
    const LENGTH_COUNTER_TABLE: [u8; 32] = [
        10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96,
        22, 192, 24, 72, 26, 16, 28, 32, 30,
    ];
    pub fn new() -> Self {
        LengthCounter {
            value: 0,
            halt: false,
            enabled: false,
        }
    }

    pub fn set_length(&mut self, value: u8) {
        if self.enabled {
            self.value = Self::LENGTH_COUNTER_TABLE[value as usize];
        }
    }

    pub fn set_halt(&mut self, halt: bool) {
        self.halt = halt;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.value = 0;
        }
    }

    pub fn get_value(&self) -> u8 {
        self.value
    }

    pub fn is_halted(&self) -> bool {
        self.halt
    }

    pub fn should_mute(&self) -> bool {
        self.value == 0 && !self.is_halted()
    }

    pub fn tick(&mut self) {
        if self.value > 0 && !self.halt {
            self.value -= 1;
        }
    }
}
