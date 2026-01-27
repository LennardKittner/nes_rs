pub struct FractionalResampler {
    accumulator: f32,
    sample_accumulator: f32,
    samples_in_accumulator: u32,
    input_rate: f32,
    output_rate: f32,
}

impl FractionalResampler {
    pub fn new(input_rate: f32, output_rate: f32) -> Self {
        Self {
            accumulator: 0f32,
            sample_accumulator: 0f32,
            samples_in_accumulator: 0,
            input_rate,
            output_rate,
        }
    }

    pub fn add_sample(&mut self, sample: f32) -> Option<f32> {
        self.accumulator += self.output_rate;
        self.sample_accumulator += sample;
        self.samples_in_accumulator += 1;

        if (self.accumulator < self.input_rate) {
            return None;
        }

        let output = self.sample_accumulator / self.samples_in_accumulator as f32;
        self.samples_in_accumulator = 0;
        self.sample_accumulator = 0f32;
        self.accumulator -= self.input_rate;

        Some(output)
    }
}
