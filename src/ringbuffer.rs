#[derive(Clone)]
pub struct RingBuffer<T> {
    buffer: Vec<T>,
    capacity: usize,
    pointer: usize,
}

impl<T: Clone> RingBuffer<T> {
    pub fn new(capacity: usize) -> RingBuffer<T> {
        RingBuffer {
            buffer: Vec::with_capacity(capacity),
            capacity,
            pointer: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.buffer.len() < self.capacity {
            self.buffer.push(value);
        } else {
            self.buffer[self.pointer] = value;
        }
        self.pointer = (self.pointer + 1) % self.capacity;
    }

    pub fn current(&self) -> &T {
        &self.buffer[self.pointer]
    }

    pub fn increment(&mut self) {
        self.pointer = (self.pointer + 1) % self.buffer.len();
    }
}

impl<T: Clone> FromIterator<T> for RingBuffer<T> {
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        let v: Vec<T> = iter.into_iter().collect();

        let mut ring_buffer = RingBuffer::new(v.len());
        for value in v {
            ring_buffer.push(value);
        }
        ring_buffer
    }
}

impl<'a, T: Clone> IntoIterator for &'a RingBuffer<T> {
    type Item = T;
    type IntoIter = RingBufferIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        RingBufferIterator::new(self)
    }
}

pub struct RingBufferIterator<'a, T> {
    buffer: &'a RingBuffer<T>,
    num_elements_left: i64,
    current_pointer: usize,
}

impl<'a, T> RingBufferIterator<'a, T> {
    pub fn new(buffer: &'a RingBuffer<T>) -> RingBufferIterator<'a, T> {
        let current_pointer = buffer.pointer % buffer.buffer.len();

        RingBufferIterator {
            buffer,
            current_pointer,
            num_elements_left: buffer.buffer.len() as i64,
        }
    }
}

impl<'a, T: Clone> Iterator for RingBufferIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.num_elements_left -= 1;
        if self.num_elements_left < 0 {
            return None;
        }

        let value = self.buffer.buffer[self.current_pointer].clone();
        self.current_pointer = (self.current_pointer + 1) % self.buffer.buffer.len();
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_less_than_capacity() {
        let mut ring_buffer: RingBuffer<i32> = RingBuffer::new(3);
        ring_buffer.push(1);
        ring_buffer.push(2);
        ring_buffer.increment();
        ring_buffer.increment();
        assert_eq!(*ring_buffer.current(), 1);
    }

    #[test]
    fn test_push_more_than_capacity() {
        let mut ring_buffer: RingBuffer<i32> = RingBuffer::new(2);
        ring_buffer.push(1);
        ring_buffer.push(2);
        ring_buffer.push(3);
        assert_eq!(ring_buffer.buffer[0], 3);
    }

    #[test]
    fn test_from_iterator() {
        let example = vec![1, 2, 3, 4];
        let mut ring_buffer: RingBuffer<i32> = example.into_iter().collect();
        assert_eq!(ring_buffer.capacity, 4);
        assert_eq!(*ring_buffer.current(), 1);
        ring_buffer.increment();
        ring_buffer.increment();
        ring_buffer.increment();
        ring_buffer.increment();
        assert_eq!(*ring_buffer.current(), 1);
    }

    #[test]
    fn test_into_iterator() {
        let example = vec![1, 2, 3, 4];
        let mut ring_buffer: RingBuffer<i32> = example.into_iter().collect();
        ring_buffer.increment();
        ring_buffer.increment();

        let iterator = ring_buffer.into_iter();
        let v: Vec<i32> = iterator.collect();
        assert_eq!(v, vec![3, 4, 1, 2]);
    }

    #[test]
    fn test_iterator_length_less_than_capacity() {
        let mut ring_buffer: RingBuffer<i32> = RingBuffer::new(4);
        ring_buffer.push(1);
        ring_buffer.push(2);
        ring_buffer.push(3);

        let iterator = ring_buffer.into_iter();
        let v: Vec<i32> = iterator.collect();
        assert_eq!(v, vec![1, 2, 3]);
    }
}
