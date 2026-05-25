use crate::{MessageType, SliceMessageEncoder, const_helpers::ConstMessage};

/// Represents a starting "hello" message that is sent to `DBus`
pub struct Hello;

impl Hello {
    const LEN: usize = 128;

    /// Encoded representation of a constant `Hello` message.
    pub const ENCODED: [u8; Self::LEN] = {
        let mut buf = [0; Self::LEN];
        let mut encoder = match SliceMessageEncoder::new(&mut buf, MessageType::MethodCall, 0) {
            Ok(encoder) => encoder,
            Err(err) => panic!("{}", err.display()),
        };
        if let Err(err) = encoder.set_path("/org/freedesktop/DBus") {
            panic!("{}", err.display());
        }
        if let Err(err) = encoder.set_member("Hello") {
            panic!("{}", err.display());
        }
        if let Err(err) = encoder.set_interface("org.freedesktop.DBus") {
            panic!("{}", err.display());
        }
        if let Err(err) = encoder.set_destination("org.freedesktop.DBus") {
            panic!("{}", err.display());
        }
        let len = match encoder.finish() {
            Ok(len) => len,
            Err(err) => panic!("{}", err.display()),
        };
        if len != Self::LEN {
            let message = ConstMessage::<96>::new();
            let Some(message) = message.push_str("buffer is too long, can be just ") else {
                panic!("failed to format buffer length error");
            };
            let Some(message) = message.push_usize(len) else {
                panic!("failed to format buffer length error");
            };
            let Some(message) = message.push_str(" bytes, not ") else {
                panic!("failed to format buffer length error");
            };
            let Some(message) = message.push_usize(Self::LEN) else {
                panic!("failed to format buffer length error");
            };
            let Some(message) = message.as_str() else {
                panic!("failed to format buffer length error");
            };
            panic!("{}", message);
        }
        buf
    };
}
