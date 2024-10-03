/// NES APU envelope generator
/// Controls the volume
pub struct EnvelopeGenerator {
    /// start flag to reset envelope
    start: bool,
    /// constant volume mode flag
    constant_volume_flag: bool,
    /// whether the envelope should be looped
    loop_envelope: bool,
    /// timer for envelope decay
    divider: u8,
    /// volume for constant volume mode
    constant_volume: u8,
    /// initial divider value
    initial_divider: u8,
    /// represents current volume when not using constant volume mode
    decay_level_counter: u8
}

impl EnvelopeGenerator {
    /// create a new EnvelopeGenerator
    pub fn new() -> Self {
        EnvelopeGenerator {
            start: false,
            constant_volume_flag: false,
            decay_level_counter: 0,
            loop_envelope: false,
            divider: 0,
            constant_volume: 0,
            initial_divider: 0,
        }
    }

    /// Tick the envelope generator
    pub fn tick(&mut self) {
        if self.constant_volume_flag {
            return;
        }

        if self.start {
            self.divider = self.initial_divider;
            self.decay_level_counter = 15;
            self.start = false;
            return;
        }

        if self.divider > 0 {
            self.divider -= 1;
            return;
        }
        
        self.divider = self.initial_divider;

        if self.decay_level_counter > 0 {
            self.decay_level_counter -= 1;
            return;
        }

        if self.loop_envelope {
            self.decay_level_counter = 15;
        }
    }

    /// set volume for constant volume mode
    pub fn set_volume(&mut self, vol: u8) {
        self.constant_volume = vol;
    }

    /// set envelope divider
    pub fn set_envelope(&mut self, envelope: u8) {
        self.initial_divider = envelope;
    }

    /// set whether the envelope should loop
    pub fn set_loop_envelope(&mut self, loop_envelope: bool) {
        self.loop_envelope = loop_envelope;
    }

    /// set the constant volume flag
    pub fn set_constant_volume(&mut self, constant_volume: bool) {
        self.constant_volume_flag = constant_volume;
    }

    /// reset envelope
    pub fn set_start(&mut self, start: bool) {
        self.start = start;
    }
    
    /// get current volume
    pub fn get_volume(&self) -> u8 {
        if self.constant_volume_flag {
            self.constant_volume
        } else {
            self.decay_level_counter
        }
    }

    /// get current normalized volume in [0, 1]
    pub fn get_volume_normalized(&self) -> f32 {
        self.get_volume() as f32 / 15.0
    }
}