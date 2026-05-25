use crate::{EncodeError, EncodeMessage, MessageType, SliceMessageEncoder};

/// Represents a starting "hello" message that is sent to `DBus`
pub struct Hello;

impl EncodeMessage for Hello {
    fn encoded_capacity(&self) -> usize {
        256
    }

    fn encode_message(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall, 0)?;
        encoder.set_path("/org/freedesktop/DBus")?;
        encoder.set_member("Hello")?;
        encoder.set_interface("org.freedesktop.DBus")?;
        encoder.set_destination("org.freedesktop.DBus")?;
        encoder.finish()
    }
}
