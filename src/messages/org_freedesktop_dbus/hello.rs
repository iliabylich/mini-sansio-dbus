use crate::{
    EncodeError, MessageType, SliceMessageEncoder, const_helpers::t_err, encode_message,
    messaging::StaticallyEncodedMessage,
};

/// `Hello` message
pub struct Hello;
impl Hello {
    const fn encode(buf: &mut [u8]) -> Result<usize, EncodeError> {
        let mut encoder = t_err!(SliceMessageEncoder::new(buf, MessageType::MethodCall));
        t_err!(encoder.set_path("/org/freedesktop/DBus"));
        t_err!(encoder.set_member("Hello"));
        t_err!(encoder.set_interface("org.freedesktop.DBus"));
        t_err!(encoder.set_destination("org.freedesktop.DBus"));
        encoder.finish()
    }
}

impl StaticallyEncodedMessage for Hello {
    const ENCODED: &[u8] = &encode_message!(128, |buf| => Self::encode(buf));
}
