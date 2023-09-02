use std::collections::{vec_deque, VecDeque};

pub struct RingBuffer<T> {
    size: usize,
    data: VecDeque<T>,
}
impl<T> RingBuffer<T> {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            data: VecDeque::with_capacity(size),
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn iter(&self) -> vec_deque::Iter<'_, T> {
        self.data.iter()
    }

    pub fn drain(&mut self) -> vec_deque::Drain<'_, T> {
        self.data.drain(0..)
    }

    pub fn push(&mut self, val: T) {
        self.data.push_back(val);
        if self.data.len() > self.size {
            self.data.pop_front();
        }
    }
}
