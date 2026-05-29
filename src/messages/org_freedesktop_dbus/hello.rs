use crate::{MessageType, SliceMessageEncoder, const_helpers::try_, def_constant_message};

def_constant_message!(name = Hello, size = 128, |buf| => {
    let mut encoder = try_!(SliceMessageEncoder::new(buf, MessageType::MethodCall, 0));
    try_!(encoder.set_path("/org/freedesktop/DBus"));
    try_!(encoder.set_member("Hello"));
    try_!(encoder.set_interface("org.freedesktop.DBus"));
    try_!(encoder.set_destination("org.freedesktop.DBus"));
    encoder.finish()
});
