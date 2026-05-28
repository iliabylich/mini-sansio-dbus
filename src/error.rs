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

impl DBusError {
    /// Returns a static string representation of the error.
    #[must_use]
    pub const fn display(self) -> &'static str {
        match self {
            Self::NoPath => "NoPath",
            Self::NoMember => "NoMember",
            Self::NoInterface => "NoInterface",
            Self::NoDestination => "NoDestination",
            Self::NoSender => "NoSender",
            Self::NoBody => "NoBody",
            Self::NoPropertyName => "NoPropertyName",
            Self::UnknownMember => "UnknownMember",
            Self::UnknownInterface => "UnknownInterface",
            Self::WrongMessageType => "WrongMessageType",
            Self::WrongDestination => "WrongDestination",
            Self::WrongInterface => "WrongInterface",
            Self::WrongPath => "WrongPath",
            Self::WrongMember => "WrongMember",
            Self::WrongValue => "WrongValue",
            Self::MalformedSignature => "MalformedSignature",
            Self::MalformedValue => "MalformedValue",
            Self::MalformedArray => "MalformedArray",
            Self::MalformedDictEntry => "MalformedDictEntry",
            Self::MalformedVariant => "MalformedVariant",
            Self::ParseError => "ParseError",
            Self::MalformedHeaderField => "MalformedHeaderField",
            Self::NoHeader => "NoHeader",
            Self::UnknownMessageType(_) => "UnknownMessageType",
            Self::MalformedBody => "MalformedBody",
            Self::MessageLengthOverflow => "MessageLengthOverflow",
            Self::InternalError => "InternalError",
            Self::UnexpectedBody => "UnexpectedBody",
            Self::NoSessionBusAddress => "NoSessionBusAddress",
            Self::MalformedSessionBusAddress => "MalformedSessionBusAddress",
            Self::DBusPathWithNull => "DBusPathWithNull",
            Self::EncodeError(encode_err) => match encode_err {
                EncodeError::BufferTooSmall => "EncodeError(BufferTooSmall)",
                EncodeError::TypeMismatch => "EncodeError(TypeMismatch)",
                EncodeError::ValueTooLong => "EncodeError(ValueTooLong)",
                EncodeError::ContainerTooLong => "EncodeError(ContainerTooLong)",
                EncodeError::HeaderAlreadyFinished => "EncodeError(HeaderAlreadyFinished)",
                EncodeError::BodySignatureExhausted => "EncodeError(BodySignatureExhausted)",
                EncodeError::BodySignatureIncomplete => "EncodeError(BodySignatureIncomplete)",
                EncodeError::ValueAlreadyWritten => "EncodeError(ValueAlreadyWritten)",
            },
            Self::ReadBufIsTooShort => "ReadBufIsTooShort",
            Self::Other(message) => message,
            Self::ErrorReply => "ErrorReply",
        }
    }
}
