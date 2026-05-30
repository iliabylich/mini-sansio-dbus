## `DBus` Sans-IO client

It doesn't implement any IO. [Examples](./examples/) directory has a simple usage example.

Outgoing message bodies are written with `dbus_body!`:

```rust
use mini_sansio_dbus::{dbus_body, MessageType, SliceMessageEncoder};

fn encode(buf: &mut [u8]) -> Result<usize, mini_sansio_dbus::EncodeError> {
    let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall)?;
    encoder.set_path("/org/example/Object")?;
    encoder.set_member("Example")?;

    dbus_body!(encoder, {
        str("hello"),
        u32(42),
        array<u16> [1, 2, 3],
    });

    encoder.finish()
}
```
