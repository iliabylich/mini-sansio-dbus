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
            1 => HeaderFieldCode::Path,
            2 => HeaderFieldCode::Interface,
            3 => HeaderFieldCode::Member,
            4 => HeaderFieldCode::ErrorName,
            5 => HeaderFieldCode::ReplySerial,
            6 => HeaderFieldCode::Destination,
            7 => HeaderFieldCode::Sender,
            8 => HeaderFieldCode::Signature,
            9 => HeaderFieldCode::UnixFds,
            _ => HeaderFieldCode::Invalid,
        }
    }
}

impl From<HeaderFieldCode> for u8 {
    fn from(header_field: HeaderFieldCode) -> Self {
        header_field as u8
    }
}
