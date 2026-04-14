// use num::{CheckedSub, Saturating};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Timer {
    pub timer_limit: u16,
    pub data: u16,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            timer_limit: 0,
            data: 0,
        }
    }

    pub fn tick(&mut self, cycles: u8) -> bool {
        if self.data >= cycles as u16 {
            self.data -= cycles as u16;
            false
        } else {
            let remaining = cycles as u16 - self.data;
            self.data = self.timer_limit.saturating_sub(remaining);
            true
        }
    }
}
