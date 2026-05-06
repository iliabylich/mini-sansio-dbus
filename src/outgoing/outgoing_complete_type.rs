/// A type of outgoing (sent) `DBus` value
#[derive(Debug, Clone, PartialEq, Eq)]
#[expect(missing_docs)]
pub enum OutgoingCompleteType {
    Byte,
    Bool,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Double,
    UnixFD,

    String,
    ObjectPath,
    Signature,
    Struct(Vec<Self>),
    Array(Box<Self>),
    DictEntry(Box<Self>, Box<Self>),
    Variant,
}

impl OutgoingCompleteType {
    pub(crate) const fn alignment(&self) -> usize {
        match self {
            Self::Byte | Self::Signature | Self::Variant => 1,

            Self::Int16 | Self::UInt16 => 2,

            Self::Bool
            | Self::Int32
            | Self::UInt32
            | Self::UnixFD
            | Self::String
            | Self::ObjectPath
            | Self::Array(_) => 4,

            Self::Int64 | Self::UInt64 | Self::Double | Self::Struct(_) | Self::DictEntry(_, _) => {
                8
            }
        }
    }
}
