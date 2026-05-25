use crate::{EncodeError, EncodeMessage, MessageType, SliceMessageEncoder, dbus_body};

/// Represents a request to `DBus` to occupy some name
pub struct RequestName<'a> {
    name: &'a str,
}

impl<'a> RequestName<'a> {
    /// Constructor for the slice-encoded message.
    #[must_use]
    pub const fn new(name: &'a str) -> Self {
        Self { name }
    }
}

impl EncodeMessage for RequestName<'_> {
    fn encoded_capacity(&self) -> usize {
        256usize.saturating_add(self.name.len())
    }

    fn encode_message(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall, 0)?;
        encoder.set_path("/org/freedesktop/DBus")?;
        encoder.set_member("RequestName")?;
        encoder.set_interface("org.freedesktop.DBus")?;
        encoder.set_destination("org.freedesktop.DBus")?;
        dbus_body!(encoder, {
            str(self.name),
            u32(7),
        });
        encoder.finish()
    }
}
