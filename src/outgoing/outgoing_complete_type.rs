#[derive(Debug, Clone, PartialEq, Eq)]
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
    Struct(Vec<OutgoingCompleteType>),
    Array(Box<OutgoingCompleteType>),
    DictEntry(Box<OutgoingCompleteType>, Box<OutgoingCompleteType>),
    Variant,
}

impl OutgoingCompleteType {
    pub(crate) fn alignment(&self) -> usize {
        match self {
            Self::Byte => 1,
            Self::Bool => 4,
            Self::Int16 => 2,
            Self::UInt16 => 2,
            Self::Int32 => 4,
            Self::UInt32 => 4,
            Self::Int64 => 8,
            Self::UInt64 => 8,
            Self::Double => 8,
            Self::UnixFD => 4,
            Self::String => 4,
            Self::ObjectPath => 4,
            Self::Signature => 1,
            Self::Struct(_) => 8,
            Self::Array(_) => 4,
            Self::DictEntry(_, _) => 8,
            Self::Variant => 1,
        }
    }
}
