# Rotating Buffer (RotBuf)

A dynamically sized Queue implementation using the `Bytes` crate's BufferMut.  The RotatingBuffer allows user to store sequenced bytes in a bytes buffer without needing to move data down the buffer.

To get started, you can easily create a `RotatingBuffer` knowing only the maximum size.  Resizing is not currently implemented but may be implemented in the future, so choose your size wisely.

```rust
use rotbuf::RotatingBuffer;

fn create_rotating_buffer() -> RotatingBuffer {
    RotatingBuffer::new(10)
}
```

## Enqueueing and Dequeueing

The simplest way to use the `RotatingBuffer` is to treat it like a queue, enqueing and dequeing one byte at a time.

`enqueue` is very easy, just provide it with any `u8` (best representation for a singular byte).

`dequeue` returns an `Option`, containing either the front most byte in Some, or, if empty, None.

```rust
rb = RotatingBuffer::new(10);
rb.enqueue(50)?
match rb.dequeue() {
    Some(value) => println!("Look, we dequeued something: {}", value),
    None => println!("Womp womp, we were empty."),
}
```

`enqueue` in most cases will return an empty [Ok] to signify it was successful.  If it reaches the capacity of the RotatingBuffer, it will return an Err with a RotatingBufferAtCapacity.  

```rust
match rb.enqueue(50) {
    Ok(()) => println!("The value was enqueued"),
    Err(err) => println!("Oh no we must be at capacity: {}", err)
}
```

The RotatingBufferAtCapacity is an Error, but you can reclaim the value you provided by using the `reclaim` fn

```rust
match rb.enqueue(50) {
    Ok(()) => println!("The value was enqueued"),
    Err(err) => println!("Oh no we couldn't enqueue this byte: {}", err.reclaim())
}
```

