# RotatingBuffer

A dynamically sized Queue implementation using the `Bytes` crate's BufferMut.  The RotatingBuffer allows user to store sequenced bytes in a bytes buffer without needing to move data down the buffer.

To get started, you can easily create a `RotatingBuffer` knowing only the maximum size.  Resizing is not currently implemented but may be implemented in the future, so choose your size wisely.

```rust
use rotatingbuffer::RotatingBuffer;

fn create_rotating_buffer() -> RotatingBuffer {
    RotatingBuffer::new(10)
}
```

The simplest way to use the `RotatingBuffer` is to treat it like a queue, enqueing and dequeing one byte at a time.

```rust

rb = RotatingBuffer::new(10);
rb.enqueue(50)?
rb.dequeue()

```

`dequeue` returns an Option, containing either the front most byte in Some, or, if empty, None.

`enqueue` on the other hand returns an error enum.  At the moment, the only error that can