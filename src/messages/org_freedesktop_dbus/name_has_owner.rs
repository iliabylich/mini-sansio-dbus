use crate::{
    DBusError, EncodeError, IncomingBody, IncomingValue, MessageType, SliceMessageEncoder,
    dbus_body,
    messaging::{DBusEncode, reply_handler::HandleReply},
    value_is,
};

/// Represents a request to check if a bus name currently has an owner.
#[derive(Clone, Copy)]
pub struct NameHasOwner;

impl DBusEncode for NameHasOwner {
    type Args<'a> = &'a str;

    fn encode<'a>(name: Self::Args<'_>, buf: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall)?;
        encoder.set_path("/org/freedesktop/DBus")?;
        encoder.set_member("NameHasOwner")?;
        encoder.set_interface("org.freedesktop.DBus")?;
        encoder.set_destination("org.freedesktop.DBus")?;
        dbus_body!(encoder, {
            str(name),
        });
        let len = encoder.finish()?;
        buf.get(0..len).ok_or(EncodeError::BufferTooSmall)
    }
}

impl HandleReply for NameHasOwner {
    type Output<'a> = bool;

    fn handle_reply_body(&self, mut body: IncomingBody<'_>) -> Result<Self::Output<'_>, DBusError> {
        let value = body
            .try_next()?
            .ok_or(DBusError::Other("NameHasOwner reply has no body"))?;
        value_is!(value, IncomingValue::Bool(value));
        Ok(value)
    }
}
