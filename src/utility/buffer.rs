use std::collections::VecDeque;

pub struct Buffer<T> {
    buffer: VecDeque<T>,
    buffer_size: usize,
}

impl<T> Buffer<T> {
    pub fn with_capacity(buffer_size: usize) -> Self {
        let buffer = VecDeque::with_capacity(buffer_size);
        Buffer {
            buffer,
            buffer_size,
        }
    }

    pub fn push_front(&mut self, value: T) {
        self.buffer.push_front(value);
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.buffer.pop_back()
    }

    pub fn is_full(&self) -> bool {
        self.buffer.len() >= self.buffer_size
    }
}
