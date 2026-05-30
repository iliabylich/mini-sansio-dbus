use crate::{
    EncodeError, MessageType, SliceMessageEncoder, const_helpers::get_range, dbus_body_fragment,
};

/// Represents a request to set a single property on a given `DBus` object
pub struct SetProperty;

impl SetProperty {
    /// Writes a "set property" message to a given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't fit into given buffer.
    pub fn encode<'a>(
        buf: &'a mut [u8],
        destination: &str,
        path: &str,
        interface: &str,
        property: &str,
        value: impl Fn(&mut SliceMessageEncoder<'_>) -> Result<(), EncodeError>,
    ) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall)?;
        encoder.set_path(path)?;
        encoder.set_member("Set")?;
        encoder.set_interface("org.freedesktop.DBus.Properties")?;
        encoder.set_destination(destination)?;
        encoder.set_body_signature("ssv")?;
        encoder.__dbus_begin_body()?;
        dbus_body_fragment!(encoder, {
            str(interface),
            str(property),
        });
        (value)(&mut encoder)?;

        let len = encoder.finish()?;
        let Some(buf) = get_range(buf, 0, len) else {
            return Err(EncodeError::BufferTooSmall);
        };
        Ok(buf)
    }
}
