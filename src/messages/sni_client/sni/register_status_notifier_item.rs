use crate::{
    DBusError, EncodeError, IncomingBody, MessageType, SliceMessageEncoder, dbus_body,
    messaging::{DBusEncode, reply_handler::HandleReply},
};

/// Represents a request to register an SNI item with the watcher.
pub struct RegisterStatusNotifierItem;

impl DBusEncode for RegisterStatusNotifierItem {
    type Args<'a> = &'a str;

    fn encode<'a>(item: Self::Args<'_>, buf: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall)?;
        encoder.set_destination("org.kde.StatusNotifierWatcher")?;
        encoder.set_path("/StatusNotifierWatcher")?;
        encoder.set_interface("org.kde.StatusNotifierWatcher")?;
        encoder.set_member("RegisterStatusNotifierItem")?;
        dbus_body!(encoder, {
            str(item),
        });
        let len = encoder.finish()?;
        buf.get(0..len).ok_or(EncodeError::BufferTooSmall)
    }
}

impl HandleReply for RegisterStatusNotifierItem {
    type Output<'a> = ();

    fn handle_reply_body(&self, _body: IncomingBody<'_>) -> Result<Self::Output<'_>, DBusError> {
        Ok(())
    }
}
