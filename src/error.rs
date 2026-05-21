/// A sum type of all possible error kinds
#[derive(Debug)]
#[expect(missing_docs)]
pub enum DBusError {
    NoPath,
    NoMember,
    NoInterface,
    NoDestination,
    NoSender,
    NoBody,
    NoPropertyName,
    UnknownMember(String),
    UnknownInterface(String),
    WrongMessageType(String),
    WrongDestination(String),
    WrongInterface(String),
    WrongPath(String),
    WrongMember(String),
    WrongSender(String),
    WrongValue(String),
    BodyEOF,
    MalformedSignature,
    MalformedValue,
    MalformedArray,
    MalformedDictEntry,
    MalformedStruct,
    MalformedVariant,
    ParseError,
    MalformedHeaderField(String),
    NoHeader,
    UnknownMessageType(u8),
    MalformedBody,
    MessageLengthOverflow,
    InternalError(String),
    ConnectError(String),
    ReadError(String),
    WriteError(String),
    UnexpectedBody,
    DBusError(String),
    NoSystemBusAddress,
    NoSessionBusAddress,
    MalformedSessionBusAddress,
    NoDataAttachedToMethodCall,
    DBusPathWithNull,
}

impl core::fmt::Display for DBusError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for DBusError {}
