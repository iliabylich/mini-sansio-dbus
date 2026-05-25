use crate::{
    DBusError, EncodeError, EncodeMessage, MessageType, SliceMessageEncoder, Str,
    incoming::IncomingMessage, interface_is, member_is, path_is,
};

/// Low-level introspection request received from `DBus`
#[derive(Debug)]
#[expect(missing_docs)]
pub struct IntrospectRequest {
    pub serial: u32,
    pub destination: String,
    pub path: String,
    pub sender: String,
}

impl TryFrom<IncomingMessage<'_>> for IntrospectRequest {
    type Error = DBusError;

    fn try_from(message: IncomingMessage) -> Result<Self, Self::Error> {
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
            destination: destination.to_string(),
            path: path.to_string(),
            sender: sender.to_string(),
        })
    }
}

/// Low-level introspection response to send to `DBus`
#[derive(Debug)]
pub struct IntrospectResponse<'a> {
    reply_serial: u32,
    destination: &'a str,
    xml: &'a str,
}

impl<'a> IntrospectResponse<'a> {
    /// Constructor for the slice-encoded message.
    #[must_use]
    pub const fn new(reply_serial: u32, destination: &'a str, xml: &'a str) -> Self {
        Self {
            reply_serial,
            destination,
            xml,
        }
    }
}

impl EncodeMessage for IntrospectResponse<'_> {
    fn encoded_capacity(&self) -> usize {
        128usize
            .saturating_add(self.destination.len())
            .saturating_add(self.xml.len())
    }

    fn encode_message(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodReturn, 0)?;
        encoder.set_reply_serial(self.reply_serial)?;
        encoder.set_destination(self.destination)?;
        encoder.set_body_signature("s")?;
        encoder.next_body_slot::<Str>()?.write(self.xml)?;
        encoder.finish()
    }
}
