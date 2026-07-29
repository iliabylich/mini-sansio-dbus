use crate::{EncodeError, MessageType, SliceMessageEncoder, dbus_body, messaging::DBusEncode};

/// Represents a request to get all object properties
pub struct GetAllProperties;

impl DBusEncode for GetAllProperties {
    type Args<'a> = (&'a str, &'a str, &'a str);

    fn encode<'a>(
        (destination, path, interface): Self::Args<'_>,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall)?;
        encoder.set_path(path)?;
        encoder.set_member("GetAll")?;
        encoder.set_interface("org.freedesktop.DBus.Properties")?;
        encoder.set_destination(destination)?;
        dbus_body!(encoder, {
            str(interface),
        });
        let len = encoder.finish()?;
        buf.get(0..len).ok_or(EncodeError::BufferTooSmall)
    }
}
