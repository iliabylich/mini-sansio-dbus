use crate::DBusError;

/// A type of a message
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
#[expect(missing_docs)]
pub enum MessageType {
    Invalid = 0,
    MethodCall = 1,
    MethodReturn = 2,
    Error = 3,
    Signal = 4,
}

impl TryFrom<u8> for MessageType {
    type Error = DBusError;

    fn try_from(value: u8) -> Result<Self, DBusError> {
        let ty = match value {
            0 => Self::Invalid,
            1 => Self::MethodCall,
            2 => Self::MethodReturn,
            3 => Self::Error,
            4 => Self::Signal,
            other => return Err(DBusError::UnknownMessageType(other)),
        };
        Ok(ty)
    }
}

impl From<MessageType> for u8 {
    fn from(message_type: MessageType) -> Self {
        message_type as Self
    }
}
