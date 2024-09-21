use std::cmp::min;
use num::{FromPrimitive, Num, ToPrimitive};

pub struct RollingAvg<T: Num + Clone + Copy + FromPrimitive> {
    buffer: Vec<T>,
    index: usize,
    count: usize,
    size: usize,
}

impl<T: Num + Clone + Copy + FromPrimitive + ToPrimitive> RollingAvg<T> {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![T::zero(); size],
            index: 0,
            count: 0,
            size
        }
    }

    pub fn push(&mut self, value: T) {
        self.buffer[self.index] = value;
        self.index = (self.index + 1) % self.size;
        self.count = min(self.count+1, self.size);
    }

    pub fn avg(&self) -> Option<T> {
        if self.count == 0 {
            return None;
        }
        let mut avg = 0f64;
        for &z in self.buffer.iter() {
            avg += z.to_f64()? / (self.count as f64);
        }
        T::from_f64(avg)
    }
}