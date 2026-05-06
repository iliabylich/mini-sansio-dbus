#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum HeaderFieldCode {
    Invalid = 0,
    Path = 1,
    Interface = 2,
    Member = 3,
    ErrorName = 4,
    ReplySerial = 5,
    Destination = 6,
    Sender = 7,
    Signature = 8,
    UnixFds = 9,
}

impl From<u8> for HeaderFieldCode {
    fn from(byte: u8) -> Self {
        match byte {
            1 => Self::Path,
            2 => Self::Interface,
            3 => Self::Member,
            4 => Self::ErrorName,
            5 => Self::ReplySerial,
            6 => Self::Destination,
            7 => Self::Sender,
            8 => Self::Signature,
            9 => Self::UnixFds,
            _ => Self::Invalid,
        }
    }
}

impl From<HeaderFieldCode> for u8 {
    fn from(header_field: HeaderFieldCode) -> Self {
        header_field as Self
    }
}
