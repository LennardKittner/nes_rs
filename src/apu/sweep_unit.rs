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
            pulse_generator_id
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
        if self.divider_value == 0 && self.enabled && self.shift != 0 {
            self.divider_value = self.divider_period;
            self.reload = false;
            return if self.should_mute(current_time) {
                current_time
            } else {
                self.calculate_period(current_time)
            }
        }

        if self.reload {
            self.divider_value = self.divider_period;
            self.reload = false;
        }

        current_time
    }

    fn calculate_period(&self, current_period: u16) -> u16 {
        let tmp = current_period.wrapping_shr(self.shift as u32);
        let upper = (tmp & 0b1111_1000_0000_0000) >> 5;
        let mut result = tmp | upper;
        if self.negate {
            //TODO: neg on 16 bit the same as neg on 11 bit?
            result = result.wrapping_neg();
            if self.pulse_generator_id == PulseGeneratorID::ONE {
                result = result.wrapping_sub(1);
            }
        }
        result.saturating_add(current_period)
    }

    pub fn should_mute(&self, current_period: u16) -> bool {
        current_period < 8 || self.calculate_period(current_period) > 0x7FF
    }
}