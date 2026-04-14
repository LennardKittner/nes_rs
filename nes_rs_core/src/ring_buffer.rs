#[derive(Debug)]
pub struct RingBuffer<T, const BUFFER_SIZE: usize> {
    buffer: [Option<T>; BUFFER_SIZE],
    pub writer_head: usize,
    pub reader_head: usize,
}

impl<T, const BUFFER_SIZE: usize> Iterator for RingBuffer<T, BUFFER_SIZE> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.writer_head > self.reader_head {
            let sample = self.buffer[self.reader_head % BUFFER_SIZE].take();
            self.reader_head += 1;
            sample
        } else {
            None
        }
    }
}

impl<T, const BUFFER_SIZE: usize> RingBuffer<T, BUFFER_SIZE> {
    pub fn new() -> RingBuffer<T, BUFFER_SIZE> {
        RingBuffer {
            buffer: [const { None }; BUFFER_SIZE],
            writer_head: 0,
            reader_head: 0,
        }
    }

    pub fn push(&mut self, data: T) {
        self.buffer[self.writer_head % BUFFER_SIZE] = Some(data);
        self.writer_head += 1;
    }

    pub fn get(&mut self, index: usize) -> Option<T> {
        self.writer_head = index;
        self.buffer[index % BUFFER_SIZE].take()
    }

    pub fn peak(&self, index: usize) -> Option<&T> {
        self.buffer[index % BUFFER_SIZE].as_ref()
    }
}

impl<T, const BUFFER_SIZE: usize> Default for RingBuffer<T, BUFFER_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}
