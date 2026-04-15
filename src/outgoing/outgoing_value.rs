use crate::OutgoingCompleteType;

#[derive(Debug, PartialEq, Clone)]
pub enum OutgoingValue {
    Byte(u8),
    Bool(bool),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Double(f64),
    UnixFD(u32),

    String(String),
    ObjectPath(String),
    Signature(Vec<u8>),
    Struct(Vec<OutgoingValue>),
    Array(OutgoingCompleteType, Vec<OutgoingValue>),
    DictEntry(Box<OutgoingValue>, Box<OutgoingValue>),
    Variant(Box<OutgoingValue>),
}

impl OutgoingValue {
    pub(crate) fn complete_type(&self) -> OutgoingCompleteType {
        match self {
            Self::Byte(_) => OutgoingCompleteType::Byte,
            Self::Bool(_) => OutgoingCompleteType::Bool,
            Self::Int16(_) => OutgoingCompleteType::Int16,
            Self::UInt16(_) => OutgoingCompleteType::UInt16,
            Self::Int32(_) => OutgoingCompleteType::Int32,
            Self::UInt32(_) => OutgoingCompleteType::UInt32,
            Self::Int64(_) => OutgoingCompleteType::Int64,
            Self::UInt64(_) => OutgoingCompleteType::UInt64,
            Self::Double(_) => OutgoingCompleteType::Double,
            Self::UnixFD(_) => OutgoingCompleteType::UnixFD,
            Self::String(_) => OutgoingCompleteType::String,
            Self::ObjectPath(_) => OutgoingCompleteType::ObjectPath,
            Self::Signature(_) => OutgoingCompleteType::Signature,
            Self::Struct(values) => {
                let mut types = vec![];
                for value in values {
                    types.push(value.complete_type());
                }
                OutgoingCompleteType::Struct(types)
            }
            Self::Array(item_type, items) => {
                for item in items {
                    if item.complete_type() != *item_type {
                        panic!("heterogenous array")
                    }
                }
                OutgoingCompleteType::Array(Box::new(item_type.clone()))
            }
            Self::DictEntry(key, value) => OutgoingCompleteType::DictEntry(
                Box::new(key.complete_type()),
                Box::new(value.complete_type()),
            ),
            Self::Variant(_value) => OutgoingCompleteType::Variant,
        }
    }
}
