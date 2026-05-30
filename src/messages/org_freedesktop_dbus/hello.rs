use crate::{MessageType, SliceMessageEncoder, const_helpers::t_err, def_constant_message};

def_constant_message!(name = Hello, size = 128, |buf| => {
    let mut encoder = t_err!(SliceMessageEncoder::new(buf, MessageType::MethodCall));
    t_err!(encoder.set_path("/org/freedesktop/DBus"));
    t_err!(encoder.set_member("Hello"));
    t_err!(encoder.set_interface("org.freedesktop.DBus"));
    t_err!(encoder.set_destination("org.freedesktop.DBus"));
    encoder.finish()
});
