use crate::{MessageType, OutgoingValue};

/// Represents a request that you send to `DBus`
#[derive(Debug, PartialEq)]
#[must_use]
#[expect(missing_docs)]
pub enum OutgoingMessage {
    MethodCall {
        destination: Option<String>,
        path: String,
        interface: Option<String>,
        serial: u32,
        member: String,
        sender: Option<String>,
        unix_fds: Option<u32>,
        body: Vec<OutgoingValue>,
    },
    MethodReturn {
        serial: u32,
        reply_serial: u32,
        destination: Option<String>,
        sender: Option<String>,
        unix_fds: Option<u32>,
        body: Vec<OutgoingValue>,
    },
    Error {
        serial: u32,
        error_name: String,
        reply_serial: u32,
        destination: Option<String>,
        sender: Option<String>,
        unix_fds: Option<u32>,
        body: Vec<OutgoingValue>,
    },
}

impl OutgoingMessage {
    /// Serial of the message
    #[must_use]
    pub const fn serial(&self) -> u32 {
        match self {
            Self::MethodCall { serial, .. }
            | Self::MethodReturn { serial, .. }
            | Self::Error { serial, .. } => *serial,
        }
    }

    /// Serial (mutable) of the message
    #[must_use]
    pub const fn serial_mut(&mut self) -> &mut u32 {
        match self {
            Self::MethodCall { serial, .. }
            | Self::MethodReturn { serial, .. }
            | Self::Error { serial, .. } => serial,
        }
    }

    /// A type
    pub const fn message_type(&self) -> MessageType {
        match self {
            Self::MethodCall { .. } => MessageType::MethodCall,
            Self::MethodReturn { .. } => MessageType::MethodReturn,
            Self::Error { .. } => MessageType::Error,
        }
    }

    /// `Path` header field
    #[must_use]
    pub const fn path(&self) -> Option<&str> {
        match self {
            Self::MethodCall { path, .. } => Some(path.as_str()),
            _ => None,
        }
    }

    /// `Member` header field
    #[must_use]
    pub const fn member(&self) -> Option<&str> {
        match self {
            Self::MethodCall { member, .. } => Some(member.as_str()),
            _ => None,
        }
    }

    /// `Interface` header field
    #[must_use]
    pub const fn interface(&self) -> Option<&str> {
        match self {
            Self::MethodCall {
                interface: Some(interface),
                ..
            } => Some(interface.as_str()),
            _ => None,
        }
    }

    /// `ErrorName` header field
    #[must_use]
    pub const fn error_name(&self) -> Option<&str> {
        match self {
            Self::Error { error_name, .. } => Some(error_name.as_str()),
            _ => None,
        }
    }

    /// `ReplySerial` header field
    #[must_use]
    pub const fn reply_serial(&self) -> Option<u32> {
        match self {
            Self::MethodReturn { reply_serial, .. } | Self::Error { reply_serial, .. } => {
                Some(*reply_serial)
            }
            Self::MethodCall { .. } => None,
        }
    }

    /// `Destination` header field
    #[must_use]
    pub const fn destination(&self) -> Option<&str> {
        match self {
            Self::MethodCall {
                destination: Some(destination),
                ..
            }
            | Self::MethodReturn {
                destination: Some(destination),
                ..
            }
            | Self::Error {
                destination: Some(destination),
                ..
            } => Some(destination.as_str()),
            _ => None,
        }
    }

    /// `Sender` header field
    #[must_use]
    pub const fn sender(&self) -> Option<&str> {
        match self {
            Self::MethodCall {
                sender: Some(sender),
                ..
            }
            | Self::MethodReturn {
                sender: Some(sender),
                ..
            }
            | Self::Error {
                sender: Some(sender),
                ..
            } => Some(sender.as_str()),
            _ => None,
        }
    }

    /// A body
    pub const fn body(&self) -> &[OutgoingValue] {
        match self {
            Self::MethodCall { body, .. }
            | Self::MethodReturn { body, .. }
            | Self::Error { body, .. } => body.as_slice(),
        }
    }

    /// `UnixFDs` header field
    #[must_use]
    pub const fn unix_fds(&self) -> Option<u32> {
        match self {
            Self::MethodCall { unix_fds, .. }
            | Self::MethodReturn { unix_fds, .. }
            | Self::Error { unix_fds, .. } => *unix_fds,
        }
    }

    /// Constructs an empty "ok" reply
    pub fn new_method_return_no_body(reply_serial: u32, destination: impl Into<String>) -> Self {
        Self::MethodReturn {
            serial: 0,
            reply_serial,
            destination: Some(destination.into()),
            sender: None,
            unix_fds: None,
            body: vec![],
        }
    }

    /// Constructs an empty "err" reply with "no such method" description
    pub fn new_err_no_method(reply_serial: u32, destination: impl Into<String>) -> Self {
        Self::Error {
            serial: 0,
            error_name: String::from("org.freedesktop.DBus.Error.UnknownMethod"),
            reply_serial,
            destination: Some(destination.into()),
            sender: None,
            unix_fds: None,
            body: vec![OutgoingValue::String(String::from("Unknown method"))],
        }
    }
}
