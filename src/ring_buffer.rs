pub struct RingBuffer<T: Default + Copy, const BUFFER_SIZE: usize> {
    buffer: [T; BUFFER_SIZE],
    writer_head: usize,
    reader_head: usize,
    current: i64,
    prev: T,
}

impl<T: Default + Copy, const BUFFER_SIZE: usize> RingBuffer<T, BUFFER_SIZE> {
    pub fn new() -> RingBuffer<T, BUFFER_SIZE> {
        RingBuffer {
            buffer: [T::default(); BUFFER_SIZE],
            writer_head: 0,
            reader_head: 0,
            current: 0,
            prev: T::default(),
        }
    }

    pub fn push(&mut self, data: T) {
        self.buffer[self.writer_head % BUFFER_SIZE] = data;
        self.writer_head += 1;
        self.current += 1;
    }

    pub fn next(&mut self) -> Option<T> {
        if self.writer_head > self.reader_head {
            self.current -= 1;
            //println!("current: {}", self.current);
            let sample = self.buffer[self.reader_head % BUFFER_SIZE];
            self.reader_head += 1;
            self.prev = sample;
            Some(sample)
        } else {
            Some(self.prev)
        }
    }

    pub fn has_next(&self) -> bool {
        self.writer_head > self.reader_head
    }
}

impl<T: Default + Copy, const BUFFER_SIZE: usize> Default for RingBuffer<T, BUFFER_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}
