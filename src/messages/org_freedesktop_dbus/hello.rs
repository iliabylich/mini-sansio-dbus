use crate::{
    EncodeError, MessageType, SliceMessageEncoder, const_helpers::get_range, messaging::DBusEncode,
};

/// `Hello` message
pub struct Hello;
impl DBusEncode for Hello {
    type Data = ();

    fn encode((): Self::Data, buf: &mut [u8]) -> Result<&[u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall)?;
        encoder.set_path("/org/freedesktop/DBus")?;
        encoder.set_member("Hello")?;
        encoder.set_interface("org.freedesktop.DBus")?;
        encoder.set_destination("org.freedesktop.DBus")?;
        let len = encoder.finish()?;
        let buf = get_range(buf, 0, len).ok_or(EncodeError::BufferTooSmall)?;
        Ok(buf)
    }
}
