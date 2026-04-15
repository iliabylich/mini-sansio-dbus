use crate::{
    DBusError, IncomingArrayValue, IncomingDictEntryValue, IncomingStructValue,
    IncomingVariantValue,
    incoming::{Cursor, IncomingCompleteType},
};

#[derive(Debug)]
pub enum IncomingValue<'a> {
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
    String(&'a str),
    ObjectPath(&'a str),
    Signature(&'a str),
    Struct(IncomingStructValue<'a>),
    Array(IncomingArrayValue<'a>),
    DictEntry(IncomingDictEntryValue<'a>),
    Variant(IncomingVariantValue<'a>),
}

impl<'a> IncomingValue<'a> {
    pub(crate) fn cut(
        cur: &mut Cursor<'a>,
        type_: IncomingCompleteType<'a>,
    ) -> Result<Self, DBusError> {
        cur.align(type_.alignment())
            .map_err(|_| DBusError::MalformedValue)?;
        let mut cur = {
            let start = cur.offset();
            let size = type_
                .bytesize(*cur)
                .map_err(|_| DBusError::MalformedValue)?;
            let buf = cur.take(size).map_err(|_| DBusError::MalformedValue)?;
            Cursor::new(buf, start)
        };
        macro_rules! cut_primitive {
            ($f:ident, $ctor:ident) => {{
                let v = cur.$f()?;
                if !cur.is_empty() {
                    return Err(DBusError::MalformedValue);
                }
                Self::$ctor(v)
            }};
        }

        let value = match type_ {
            IncomingCompleteType::Byte => cut_primitive!(cut_u8, Byte),
            IncomingCompleteType::Bool => cut_primitive!(cut_bool, Bool),
            IncomingCompleteType::Int16 => cut_primitive!(cut_i16, Int16),
            IncomingCompleteType::UInt16 => cut_primitive!(cut_u16, UInt16),
            IncomingCompleteType::Int32 => cut_primitive!(cut_i32, Int32),
            IncomingCompleteType::UInt32 => cut_primitive!(cut_u32, UInt32),
            IncomingCompleteType::Int64 => cut_primitive!(cut_i64, Int64),
            IncomingCompleteType::UInt64 => cut_primitive!(cut_u64, UInt64),
            IncomingCompleteType::Double => cut_primitive!(cut_f64, Double),
            IncomingCompleteType::UnixFD => cut_primitive!(cut_u32, UnixFD),
            IncomingCompleteType::String => cut_primitive!(cut_string, String),
            IncomingCompleteType::ObjectPath => cut_primitive!(cut_string, ObjectPath),
            IncomingCompleteType::Signature => cut_primitive!(cut_signature, Signature),
            IncomingCompleteType::Struct { .. } => {
                Self::Struct(IncomingStructValue::new(type_, cur))
            }
            IncomingCompleteType::Array {
                item: item_type, ..
            } => Self::Array(IncomingArrayValue::cut(&mut cur, item_type)?),
            IncomingCompleteType::DictEntry {
                key: key_type,
                value: value_type,
                ..
            } => Self::DictEntry(IncomingDictEntryValue::cut(&mut cur, key_type, value_type)?),
            IncomingCompleteType::Variant => Self::Variant(IncomingVariantValue::new(cur)),
        };

        Ok(value)
    }

    pub fn log(&self, indent: usize) -> Result<(), DBusError> {
        let offset = " ".repeat(indent);

        match self {
            Self::Byte(n) => eprintln!("{offset}u8: {n}"),
            Self::Bool(bool) => eprintln!("{offset}bool: {bool}"),
            Self::Int16(n) => eprintln!("{offset}i16: {n}"),
            Self::UInt16(n) => eprintln!("{offset}u16: {n}"),
            Self::Int32(n) => eprintln!("{offset}i32: {n}"),
            Self::UInt32(n) => eprintln!("{offset}u32: {n}"),
            Self::Int64(n) => eprintln!("{offset}i64: {n}"),
            Self::UInt64(n) => eprintln!("{offset}u64: {n}"),
            Self::Double(n) => eprintln!("{offset}double: {n}"),
            Self::UnixFD(n) => eprintln!("{offset}unixfd: {n}"),
            Self::String(s) => eprintln!("{offset}string: {s:?}"),
            Self::ObjectPath(path) => eprintln!("{offset}path: {path:?}"),
            Self::Signature(signature) => eprintln!("{offset}signature: {signature:?}"),
            Self::Struct(struct_) => {
                let mut iter = struct_.iter()?;
                eprintln!("{offset}struct:");
                while let Some(item) = iter.try_next()? {
                    item.log(indent + 4)?;
                }
            }
            Self::Array(array) => {
                let mut iter = array.iter();
                eprintln!("{offset}array:");
                while let Some(item) = iter.try_next()? {
                    item.log(indent + 4)?;
                }
            }
            Self::DictEntry(pair) => {
                eprintln!("{offset}dict:");
                let (key, value) = pair.key_value()?;
                eprintln!("{offset}    key:");
                key.log(indent + 8)?;
                eprintln!("{offset}    value:");
                value.log(indent + 8)?;
            }
            Self::Variant(variant) => {
                eprintln!("{offset}variant:");
                let value = variant.materialize()?;
                value.log(indent + 4)?;
            }
        }

        Ok(())
    }
}
