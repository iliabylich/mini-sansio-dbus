use crate::{
    DBusError, MessageType, OutgoingMessage, OutgoingValue, incoming::IncomingMessage,
    interface_is, member_is, path_is,
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
            return Err(DBusError::WrongMessageType(format!(
                "expected: {:?}, got: {:?}",
                MessageType::MethodCall,
                message.message_type
            )));
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
pub struct IntrospectResponse;

impl IntrospectResponse {
    /// constructor
    pub fn build(
        reply_serial: u32,
        destination: impl Into<String>,
        xml: impl Into<String>,
    ) -> OutgoingMessage {
        OutgoingMessage::MethodReturn {
            serial: 0,
            reply_serial,
            destination: Some(destination.into()),
            sender: None,
            unix_fds: None,
            body: vec![OutgoingValue::String(xml.into())],
        }
    }
}
