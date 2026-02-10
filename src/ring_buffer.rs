pub struct RingBuffer<T: Default + Copy, const BUFFER_SIZE: usize> {
    buffer: [T; BUFFER_SIZE],
    writer_head: usize,
    reader_head: usize,
    prev: T,
}

impl<T: Default + Copy, const BUFFER_SIZE: usize> Iterator for RingBuffer<T, BUFFER_SIZE> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.writer_head > self.reader_head {
            let sample = self.buffer[self.reader_head % BUFFER_SIZE];
            self.reader_head += 1;
            self.prev = sample;
            Some(sample)
        } else {
            None
        }
    }
}

impl<T: Default + Copy, const BUFFER_SIZE: usize> RingBuffer<T, BUFFER_SIZE> {
    pub fn new() -> RingBuffer<T, BUFFER_SIZE> {
        RingBuffer {
            buffer: [T::default(); BUFFER_SIZE],
            writer_head: 0,
            reader_head: 0,
            prev: T::default(),
        }
    }

    pub fn push(&mut self, data: T) {
        self.buffer[self.writer_head % BUFFER_SIZE] = data;
        self.writer_head += 1;
    }
}

impl<T: Default + Copy, const BUFFER_SIZE: usize> Default for RingBuffer<T, BUFFER_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}
