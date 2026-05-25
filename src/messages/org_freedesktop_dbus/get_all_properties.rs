use crate::{EncodeError, EncodeMessage, MessageType, SliceMessageEncoder, dbus_body};

/// Represents a request to get all object properties
pub struct GetAllProperties<'a> {
    destination: &'a str,
    path: &'a str,
    interface: &'a str,
}

impl<'a> GetAllProperties<'a> {
    /// Constructor for the slice-encoded message.
    #[must_use]
    pub const fn new(destination: &'a str, path: &'a str, interface: &'a str) -> Self {
        Self {
            destination,
            path,
            interface,
        }
    }
}

impl EncodeMessage for GetAllProperties<'_> {
    fn encoded_capacity(&self) -> usize {
        256usize
            .saturating_add(self.destination.len())
            .saturating_add(self.path.len())
            .saturating_add(self.interface.len())
    }

    fn encode_message(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall, 0)?;
        encoder.set_path(self.path)?;
        encoder.set_member("GetAll")?;
        encoder.set_interface("org.freedesktop.DBus.Properties")?;
        encoder.set_destination(self.destination)?;
        dbus_body!(encoder, {
            str(self.interface),
        });
        encoder.finish()
    }
}
