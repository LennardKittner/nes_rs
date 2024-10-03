// use num::{CheckedSub, Saturating};

// enum TimerMode {
//     INCREASING,
//     DECREASING,
// }
//
// pub struct Timer {
//     timer_limit: u16,
//     timer_mode: TimerMode,
//     pub data: u16
// }
//
// impl Timer {
//     pub fn new() -> Self {
//         Timer {
//             timer_limit: 0,
//             timer_mode: TimerMode::INCREASING,
//             data: 0,
//         }
//     }
//
//     pub fn tick(&mut self, cycles: u8) {
//         let remaining = match self.timer_mode {
//             TimerMode::INCREASING => {
//                 if self.timer_limit < self.data + cycles as u16 {
//                     let remaining =  self.data + cycles as u16 - self.timer_limit;
//                     self.data = self.timer_limit;
//                     self.timer_mode = TimerMode::DECREASING;
//                     remaining
//                 } else {
//                     self.data += cycles as u16;
//                     0
//                 }
//             },
//             TimerMode::DECREASING => {
//                 if cycles as u16 > self.data {
//                     let remaining =  cycles as u16 - self.data;
//                     self.data = 0;
//                     self.timer_mode = TimerMode::INCREASING;
//                     remaining
//                 } else {
//                     self.data -= cycles as u16;
//                     0
//                 }
//             }
//         };
//
//         self.data = match self.timer_mode {
//             TimerMode::INCREASING => self.data + remaining,
//             TimerMode::DECREASING => self.data - remaining,
//         }
//     }
// }
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
