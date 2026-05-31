use crate::DBusError;

/// A type of a message
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum MessageType {
    /// Invalid message
    Invalid = 0,
    /// Method call
    MethodCall = 1,
    /// Method return
    MethodReturn = 2,
    /// Error
    Error = 3,
    /// Signal
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
