use crate::{EncodeError, MessageType, SliceMessageEncoder, messaging::DBusEncode};

/// `Hello` message
pub struct Hello;
impl DBusEncode for Hello {
    type Data = ();

    fn encode((): Self::Data, buf: &mut [u8]) -> Result<usize, EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall)?;
        encoder.set_path("/org/freedesktop/DBus")?;
        encoder.set_member("Hello")?;
        encoder.set_interface("org.freedesktop.DBus")?;
        encoder.set_destination("org.freedesktop.DBus")?;
        encoder.finish()
    }
}
