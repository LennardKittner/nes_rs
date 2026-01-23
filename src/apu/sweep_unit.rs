use crate::apu::pulse_generator::PulseGeneratorID;

pub struct SweepUnit {
    enabled: bool,
    divider_period: u8,
    divider_value: u8,
    negate: bool,
    shift: u8,
    reload: bool,
    pulse_generator_id: PulseGeneratorID,
}

impl SweepUnit {
    pub fn new(pulse_generator_id: PulseGeneratorID) -> Self {
        SweepUnit {
            enabled: false,
            divider_period: 0,
            divider_value: 0,
            negate: false,
            shift: 0,
            reload: false,
            pulse_generator_id,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.reload = true;
    }

    pub fn set_divider_period(&mut self, divider: u8) {
        self.divider_period = divider + 1;
        self.reload = true;
    }

    pub fn set_negate(&mut self, negate: bool) {
        self.negate = negate;
        self.reload = true;
    }

    pub fn set_enable(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.reload = true;
    }

    pub fn set_shift(&mut self, shift: u8) {
        self.shift = shift;
        self.reload = true;
    }

    pub fn tick(&mut self, current_time: u16) -> u16 {
        if self.reload {
            self.divider_value = self.divider_period;
            self.reload = false;
            return current_time;
        } else if self.divider_value > 0 {
            self.divider_value -= 1;
        }
        if self.divider_value == 0 && self.enabled && self.shift != 0 {
            self.divider_value = self.divider_period;
            if !self.should_mute(current_time) {
                return self.calculate_period(current_time);
            }
        }

        current_time
    }

    fn calculate_period(&self, current_period: u16) -> u16 {
        let change_amount = current_period >> self.shift;

        if self.negate {
            let negate = current_period.saturating_sub(change_amount);
            if self.pulse_generator_id == PulseGeneratorID::One {
                negate.saturating_sub(1)
            } else {
                negate
            }
        } else {
            current_period.wrapping_add(change_amount)
        }
    }

    pub fn should_mute(&self, current_period: u16) -> bool {
        current_period < 8 || self.calculate_period(current_period) > 0x7FF
    }
}
