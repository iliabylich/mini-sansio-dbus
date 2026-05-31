use crate::EncodeError;

/// A sum type of all possible error kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(missing_docs)]
#[must_use]
pub enum DBusError {
    NoPath,
    NoMember,
    NoInterface,
    NoDestination,
    NoSender,
    NoBody,
    NoPropertyName,
    UnknownMember,
    UnknownInterface,
    WrongMessageType,
    WrongDestination,
    WrongInterface,
    WrongPath,
    WrongMember,
    WrongValue,
    MalformedSignature,
    MalformedValue,
    MalformedArray,
    MalformedDictEntry,
    MalformedVariant,
    ParseError,
    MalformedHeaderField,
    NoHeader,
    UnknownMessageType(u8),
    MalformedBody,
    MessageLengthOverflow,
    InternalError,
    UnexpectedBody,
    NoSessionBusAddress,
    MalformedSessionBusAddress,
    DBusPathWithNull,
    EncodeError(EncodeError),
    ReadBufIsTooShort,
    Other(&'static str),
    ErrorReply,
}

impl core::fmt::Display for DBusError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for DBusError {}

impl From<EncodeError> for DBusError {
    fn from(error: EncodeError) -> Self {
        Self::EncodeError(error)
    }
}
