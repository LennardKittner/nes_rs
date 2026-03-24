use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Clone, Copy, Serialize, Deserialize, Debug)]
    pub struct ControllerButtons: u8 {
        const RIGHT  = 0b1000_0000;
        const LEFT   = 0b0100_0000;
        const DOWN   = 0b0010_0000;
        const UP     = 0b0001_0000;
        const START  = 0b0000_1000;
        const SELECT = 0b0000_0100;
        const B      = 0b0000_0010;
        const A      = 0b0000_0001;
    }
}
#[derive(Clone, Copy)]
pub enum ControllerInput {
    Controller1(bool, ControllerButtons),
    Controller2(bool, ControllerButtons),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Controller {
    strobe: bool,
    button_index: u8,
    button_state: ControllerButtons,
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            strobe: false,
            button_index: 0,
            button_state: ControllerButtons::empty(),
        }
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = data & 1 == 1;
        if self.strobe {
            self.button_index = 0;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_index > 7 {
            return 1;
        }
        let response = (self.button_state.bits() & (1 << self.button_index)) >> self.button_index;
        if !self.strobe && self.button_index <= 7 {
            self.button_index += 1;
        }
        response
    }

    pub fn set_button_state(&mut self, pressed: bool, button: ControllerButtons) {
        self.button_state.set(button, pressed);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_strobe_mode() {
        let mut controller = Controller::new();
        controller.write(1);
        controller.set_button_state(true, ControllerButtons::A);
        for _x in 0..10 {
            assert_eq!(controller.read(), 1);
        }
    }

    #[test]
    fn test_strobe_mode_on_off() {
        let mut controller = Controller::new();

        controller.write(0);
        controller.set_button_state(true, ControllerButtons::RIGHT);
        controller.set_button_state(true, ControllerButtons::LEFT);
        controller.set_button_state(true, ControllerButtons::SELECT);
        controller.set_button_state(true, ControllerButtons::B);

        for _ in 0..=4 {
            assert_eq!(controller.read(), 0);
            assert_eq!(controller.read(), 1);
            assert_eq!(controller.read(), 1);
            assert_eq!(controller.read(), 0);
            assert_eq!(controller.read(), 0);
            assert_eq!(controller.read(), 0);
            assert_eq!(controller.read(), 1);
            assert_eq!(controller.read(), 1);

            for _x in 0..10 {
                assert_eq!(controller.read(), 1);
            }
            controller.write(1);
            controller.write(0);
        }
    }
}
