//! # AirupFX Collections

use std::collections::{vec_deque, VecDeque};

/// A fixed-size ring buffer.
pub struct RingBuffer<T> {
    size: usize,
    data: VecDeque<T>,
}
impl<T> RingBuffer<T> {
    /// Creates a new [RingBuffer] instances.
    pub fn new(size: usize) -> Self {
        Self {
            size,
            data: VecDeque::with_capacity(size),
        }
    }

    /// Removes all elements from the ring buffer.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn iter(&self) -> vec_deque::Iter<'_, T> {
        self.data.iter()
    }

    pub fn drain(&mut self) -> vec_deque::Drain<'_, T> {
        self.data.drain(0..)
    }

    /// Pushes a new element to the ring buffer.
    pub fn push(&mut self, val: T) {
        self.data.push_back(val);
        if self.data.len() > self.size {
            self.data.pop_front();
        }
    }
}
