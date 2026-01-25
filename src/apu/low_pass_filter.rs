pub struct LowPassFilter {
    pub prev: f32,
    pub alpha: f32,
}

impl LowPassFilter {
    pub fn from_cutoff(sample_rate: f32, cutoff_hz: f32) -> Self {
        let rc = 1f32 / (2f32 * std::f32::consts::PI * cutoff_hz);
        let dt = 1f32 / sample_rate;
        let alpha = dt / (rc + dt);

        Self { prev: 0f32, alpha }
    }
    pub fn process(&mut self, sample: f32) -> f32 {
        self.prev = self.prev * (1f32 - self.alpha) + sample * self.alpha;
        self.prev
    }
}
