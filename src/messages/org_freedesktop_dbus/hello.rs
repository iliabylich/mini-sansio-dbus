use crate::{
    EncodeError, MessageType, SliceMessageEncoder, const_helpers::try_, def_constant_message,
};

/// Represents a starting "hello" message that is sent to `DBus`
pub struct Hello;

impl Hello {
    const fn encode(buf: &mut [u8]) -> Result<usize, EncodeError> {
        let mut encoder = try_!(SliceMessageEncoder::new(buf, MessageType::MethodCall, 0));
        try_!(encoder.set_path("/org/freedesktop/DBus"));
        try_!(encoder.set_member("Hello"));
        try_!(encoder.set_interface("org.freedesktop.DBus"));
        try_!(encoder.set_destination("org.freedesktop.DBus"));
        encoder.finish()
    }
}

def_constant_message!(Hello, 128);
