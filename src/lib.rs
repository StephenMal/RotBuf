//! Hello

use bytes::{BufMut, BytesMut};

#[derive(Debug)]
pub struct RotatingBuffer {
    buffer: BytesMut,
    head: usize,
    tail: usize,
    size: usize,
    at_capacity: bool
}

impl RotatingBuffer {
    /// Provides a partial, and invalid default struct in order to
    fn partial_default() -> Self {
        Self {
            buffer: BytesMut::new(),
            head: 0,
            tail: 0,
            size: 0,
            at_capacity: false
        }
    }

    /// Creates a new RotatingBuffer
    ///
    /// # PANICS
    ///
    /// Panics if the size is less than 2.
    pub fn new(size: usize) -> Self {
        if size <= 2 {
            panic!("Cannot create a RotatingBuffer with 2 elements or less.");
        }

        Self {
            buffer: BytesMut::with_capacity(size),
            size,
            ..Self::partial_default()
        }
    }

    fn tail(&self) -> usize {
        self.tail
    }

    fn last_indx(&self) -> Option<usize> {
        if !self.is_empty() {
            Some(self.tail() - 1)
        } else {
            None
        }
    }

    fn head(&self) -> usize {
        self.head
    }

    fn set_head(&mut self, head: usize) {
        #[cfg(debug_assertions)]
        if head >= self.size {
            unreachable!("Head should always be less than the size")
        }
        self.head = head;
    }

    fn set_tail(&mut self, tail: usize) {
        #[cfg(debug_assertions)]
        if tail >= self.size {
            unreachable!("Tail should always be less than the size")
        }

        self.tail = tail;
    }

    fn first_indx(&self) -> Option<usize> {
        if self.is_empty() {
            None
        } else {
            Some(self.head())
        }
    }

    /// Returns the index in the RotatingBuffer given the position
    fn get_index(&self, pos: usize) -> usize {
        (pos + self.head) % self.size
    }

    /// Returns a value from the index
    fn get_from_index(&self, index: usize) -> Option<u8> {
        self.buffer.get(index).copied()
    }

    /// Increments the head.
    ///
    /// ## DEBUG PANIC
    /// With `debug_assertions`, will perform a check to make sure it is not equal to tail first.
    pub(crate) fn incr_head(&mut self) {
        self.set_head((self.head + 1) % self.size);
    }

    pub(crate) fn prev_head(&self) -> usize {
        match self.head() {
            0 => self.size - 1,
            n => n - 1,
        }
    }

    /// Increments the tail
    pub(crate) fn incr_tail(&mut self) {
        #[cfg(debug_assertions)]
        if self.head() == self.tail() && self.at_capacity() {
            unreachable!("Cannot increment tail as it is at the head (full capacity)");
        }
        self.set_tail((self.tail + 1) % self.size)
    }

    /// Returns whether or not it is empty.
    pub fn is_empty(&self) -> bool {
        self.tail() == self.head() && !self.at_capacity()
    }

    /// Returns the total capacity.
    pub fn capacity(&self) -> usize {
        self.size
    }

    /// Returns the number of elements in the Queue
    pub fn len(&self) -> usize {
        match (self.tail(), self.head()) {
            (tail, head) if tail > head => tail - head,
            (tail, head) if tail < head => (self.size - head) + tail,
            // If head is at tail, then we are either empty or full.
            (tail, head) if tail == head => {
                if self.at_capacity() {
                    self.size
                } else {
                    0
                }
            },
            (tail, head) => {
                unreachable!("`tail` ({}) must by >, <, or == to `head` ({})", tail, head)
            }
        }
    }

    pub fn peek_pos(&self, pos: usize) -> Option<u8> {
        match (pos, self.len()) {
            (0, _) => self.peek(),
            (pos, len) if pos == len - 1 => self.peek_last(),
            (pos, len) if pos < len => self.get_from_index(self.get_index(pos)),
            (pos, len) if pos >= len => None,
            (pos, len) => unreachable!("`pos` ({}) must be < or >= to `len` ({}) always", pos, len),
        }
    }

    pub fn peek(&self) -> Option<u8> {
        self.get_from_index(self.first_indx()?)
    }

    pub fn peek_last(&self) -> Option<u8> {
        self.get_from_index(self.last_indx()?)
    }

    /// Returns either Some(value) or None if empty
    pub fn dequeue(&mut self) -> Option<u8> {
        match self.get_from_index(self.first_indx()?) {
            Some(value) => {
                // Increment the head
                self.incr_head();
                // Make sure at_capacity is false, because if it was true, we just cleared it.
                self.at_capacity = false;
                Some(value)
            }
            None => {
                unreachable!("If not empty, should be able to dequeue");
            }
        }
    }

    fn set_value(&mut self, index: usize, value: u8) {
        match (index, self.buffer.len()) {
            (index, len) if index == len => {
                self.buffer.put_u8(value);
            }
            (index, len) if index < len => {
                self.buffer[index] = value;
            }
            (index, len) => {
                panic!("We should never be setting values more than the current allocated buffer len ({}, {})", index, len);
            }
        }
    }

    pub fn at_capacity(&self) -> bool {
        match self.at_capacity {
            #[cfg(debug_assertions)]
            true if self.tail() != self.head() => unreachable!("at capacity is true and shouldn't be"),
            boolean => boolean
        }
    }

    /// Enqueues an item into the [RotatingBuffer].  Returns an Error if at capacity.
    pub fn enqueue(&mut self, value: u8) -> Result<(), RotatingBufferAtCapacity> {
        // If we are at capacity, return error, otherwise add tail
        if self.at_capacity() {
            Err(RotatingBufferAtCapacity(value))
        } else {
            // Retrieve the tail at current state
            let tail = self.tail();
            // If this is the last spot, then set the at_capacity boolean
            if tail == self.prev_head() {
                self.at_capacity = true;
            }
            // Set the value and increment the tail.
            self.set_value(tail, value);
            self.incr_tail();
            // Return okay
            Ok(())
        }

        
    }
}


/// [RotatingBufferAtCapacity] is a struct that represents an error.  It is returned whenever 
/// there was an attempt to enqueue a [RotatingBuffer] despite it being at capacity.  In
/// this instance, the value given is returned to the user, and can be reclaimed using
/// [RotatingBufferAtCapacity::reclaim].
#[derive(Debug)]
pub struct RotatingBufferAtCapacity(u8);

impl RotatingBufferAtCapacity {
    /// Returns the inputted value.
    pub fn reclaim(&self) -> u8 {
        self.0
    }
}

impl std::fmt::Display for RotatingBufferAtCapacity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RotatingBuffer at capacity, returned input: `{}`", self.0)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    #[should_panic(expected = "Cannot create a RotatingBuffer with 2 elements or less.")]
    fn test_new_panics_empty() {
        let _rb = RotatingBuffer::new(0);
    }

    #[test]
    #[should_panic(expected = "Cannot create a RotatingBuffer with 2 elements or less.")]
    fn test_new_panics_with_small_size_1() {
        let _rb = RotatingBuffer::new(1);
    }

    #[test]
    #[should_panic(expected = "Cannot create a RotatingBuffer with 2 elements or less.")]
    fn test_new_panics_with_small_size_2() {
        let _rb = RotatingBuffer::new(2);
    }

    #[test]
    fn test_enqueue_dequeue() {
        let mut rb = RotatingBuffer::new(3);
        rb.enqueue(1).unwrap();
        rb.enqueue(2).unwrap();
        assert_eq!(rb.dequeue(), Some(1));
        assert_eq!(rb.dequeue(), Some(2));
        assert!(rb.is_empty());
    }

    #[test]
    fn test_enqueue_at_capacity() {
        let mut rb = RotatingBuffer::new(3);
        rb.enqueue(1).unwrap();
        rb.enqueue(2).unwrap();
        rb.enqueue(3).unwrap();
        match rb.enqueue(4) {
            Ok(_) => panic!("Should have been at capacity"),
            Err(RotatingBufferAtCapacity(4)) => (),
            Err(err) => panic!("Unexpected error: {}", err),
        }
    }

    #[test]
    fn test_peek_last_functions() {
        let mut rb = RotatingBuffer::new(3);
        rb.enqueue(1).unwrap();
        rb.enqueue(2).unwrap();
        assert_eq!(rb.peek_last(), Some(2));
    }

    #[test]
    fn test_peek_first_functions() {
        let mut rb = RotatingBuffer::new(3);
        rb.enqueue(1).unwrap();
        rb.enqueue(2).unwrap();
        assert_eq!(rb.peek(), Some(1))
    }

    #[test]
    fn test_peek_at_functions() {
        let mut rb = RotatingBuffer::new(3);
        rb.enqueue(1).unwrap();
        rb.enqueue(2).unwrap();
        assert_eq!(rb.peek_pos(0), Some(1));
        assert_eq!(rb.peek_pos(1), Some(2));
    }

    #[test]
    fn test_peek_functions() {
        let mut rb = RotatingBuffer::new(3);
        rb.enqueue(1).unwrap();
        rb.enqueue(2).unwrap();
        assert_eq!(rb.peek(), Some(1));
        assert_eq!(rb.peek_pos(1), Some(2));
        assert_eq!(rb.peek_last(), Some(2));
    }

    #[test]
    fn test_len() {
        let mut rb = RotatingBuffer::new(3);
        assert_eq!(rb.len(), 0);
        rb.enqueue(0).unwrap();
        assert_eq!(rb.len(), 1);
        rb.enqueue(0).unwrap();
        assert_eq!(rb.len(), 2);
        rb.dequeue();
        assert_eq!(rb.len(), 1);
        rb.dequeue();
        assert_eq!(rb.len(), 0);
        rb.dequeue();
        assert_eq!(rb.len(), 0);
    }

    #[test]
    fn test_len_wrapped() {
        let mut rb = RotatingBuffer::new(3);
        assert_eq!(rb.len(), 0);
        rb.enqueue(1).unwrap();
        assert_eq!(rb.len(), 1);
        rb.enqueue(2).unwrap();
        assert_eq!(rb.len(), 2);
        rb.enqueue(3).unwrap();
        assert_eq!(rb.len(), 3);
        rb.dequeue();
        assert_eq!(rb.len(), 2);
        rb.dequeue();
        assert_eq!(rb.len(), 1);
        rb.dequeue();
        assert_eq!(rb.len(), 0);
        rb.enqueue(4).unwrap();
        assert_eq!(rb.len(), 1);
        rb.enqueue(5).unwrap();
        assert_eq!(rb.len(), 2);
        rb.enqueue(6).unwrap();

    }

    #[test]
    fn test_wrapping() {
        let mut rb = RotatingBuffer::new(3);
        rb.enqueue(1).unwrap();
        rb.enqueue(2).unwrap();
        rb.dequeue().unwrap(); // Remove 1
        rb.enqueue(3).unwrap();
        rb.enqueue(4).unwrap(); // This should wrap around
        assert_eq!(rb.dequeue(), Some(2));
        assert_eq!(rb.dequeue(), Some(3));
        assert_eq!(rb.dequeue(), Some(4));
    }
}
