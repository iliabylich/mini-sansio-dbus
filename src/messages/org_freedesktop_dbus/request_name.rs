use crate::{EncodeError, MessageType, SliceMessageEncoder, dbus_body, messaging::DBusEncode};

/// Represents a request to `DBus` to occupy some name
pub struct RequestName;

impl DBusEncode for RequestName {
    type Args<'a> = &'a str;

    fn encode<'a>(name: Self::Args<'_>, buf: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall)?;
        encoder.set_path("/org/freedesktop/DBus")?;
        encoder.set_member("RequestName")?;
        encoder.set_interface("org.freedesktop.DBus")?;
        encoder.set_destination("org.freedesktop.DBus")?;
        dbus_body!(encoder, {
            str(name),
            u32(7),
        });
        let len = encoder.finish()?;
        buf.get(0..len).ok_or(EncodeError::BufferTooSmall)
    }
}
