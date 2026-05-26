use crate::{
    DBusError, EncodeError, MessageType, SliceMessageEncoder, const_helpers::try_, dbus_body,
    incoming::IncomingMessage, interface_is, member_is, path_is,
};

/// Low-level introspection request received from `DBus`
#[derive(Debug)]
pub struct IntrospectRequest<'a> {
    /// Serial of the request
    pub serial: u32,
    /// Destination of the request
    pub destination: &'a str,
    /// Path of the request
    pub path: &'a str,
    /// Sender of the request
    pub sender: &'a str,
}

impl<'a> TryFrom<IncomingMessage<'a>> for IntrospectRequest<'a> {
    type Error = DBusError;

    fn try_from(message: IncomingMessage<'a>) -> Result<Self, Self::Error> {
        if message.message_type != MessageType::MethodCall {
            return Err(DBusError::WrongMessageType);
        }

        let serial = message.serial;
        let path = message.path.ok_or(DBusError::NoPath)?;
        let member = message.member.ok_or(DBusError::NoMember)?;
        let interface = message.interface.ok_or(DBusError::NoInterface)?;
        let destination = message.destination.ok_or(DBusError::NoDestination)?;
        let sender = message.sender.ok_or(DBusError::NoSender)?;

        if message.body.is_some() {
            return Err(DBusError::UnexpectedBody);
        }

        path_is!(path, "/");
        member_is!(member, "Introspect");
        interface_is!(interface, "org.freedesktop.DBus.Introspectable");

        Ok(Self {
            serial,
            destination,
            path,
            sender,
        })
    }
}

/// Low-level introspection response to send to `DBus`
pub struct IntrospectResponse;

impl IntrospectResponse {
    /// Writes an introspection response message to a given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't fit into given buffer.
    pub const fn encode(
        buf: &mut [u8],
        reply_serial: u32,
        destination: &str,
        xml: &str,
    ) -> Result<usize, EncodeError> {
        let mut encoder = try_!(SliceMessageEncoder::new(buf, MessageType::MethodReturn, 0));
        try_!(encoder.set_reply_serial(reply_serial));
        try_!(encoder.set_destination(destination));
        dbus_body!(encoder, {
            str(xml),
        });
        encoder.finish()
    }
}
