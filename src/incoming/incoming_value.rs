use crate::{
    DBusError, IncomingArrayValue, IncomingDictEntryValue, IncomingStructValue,
    IncomingVariantValue,
    incoming::{Cursor, IncomingCompleteType},
};

/// Represents an abstract received value
#[derive(Debug, Clone, Copy)]
#[must_use]
pub enum IncomingValue<'a> {
    /// A single byte number
    Byte(u8),
    /// Boolean
    Bool(bool),
    /// Signed 2-byte number
    Int16(i16),
    /// Unsigned 2-byte number
    UInt16(u16),
    /// Signed 4-byte number
    Int32(i32),
    /// Unsigned 4-byte number
    UInt32(u32),
    /// Signed 8-byte number
    Int64(i64),
    /// Unsigned 8-byte number
    UInt64(u64),
    /// Double precision float
    Double(f64),
    /// UNIX file description
    UnixFD(u32),
    /// String
    String(&'a str),
    /// Object Path
    ObjectPath(&'a str),
    /// Signature
    Signature(&'a str),
    /// Struct
    Struct(IncomingStructValue<'a>),
    /// Array
    Array(IncomingArrayValue<'a>),
    /// Dict entry
    DictEntry(IncomingDictEntryValue<'a>),
    /// Dynamic variant
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
            } => Self::DictEntry(IncomingDictEntryValue::cut(&cur, key_type, value_type)?),
            IncomingCompleteType::Variant => Self::Variant(IncomingVariantValue::new(cur)),
        };

        Ok(value)
    }

    /// Prints `self` to stderr
    ///
    /// # Errors
    ///
    /// Returns an error if some parts of the message that are parsed in a lazy manner are invalid
    pub fn log(
        &self,
        w: &mut impl core::fmt::Write,
        indent: usize,
    ) -> Result<(), core::fmt::Error> {
        fn write_indent(indent: usize, w: &mut impl core::fmt::Write) -> core::fmt::Result {
            for _ in 0..indent {
                write!(w, " ")?;
            }
            Ok(())
        }

        match self {
            Self::Byte(n) => {
                write_indent(indent, w)?;
                writeln!(w, "u8: {n}")?;
            }
            Self::Bool(bool) => {
                write_indent(indent, w)?;
                writeln!(w, "bool: {bool}")?;
            }
            Self::Int16(n) => {
                write_indent(indent, w)?;
                writeln!(w, "i16: {n}")?;
            }
            Self::UInt16(n) => {
                write_indent(indent, w)?;
                writeln!(w, "u16: {n}")?;
            }
            Self::Int32(n) => {
                write_indent(indent, w)?;
                writeln!(w, "i32: {n}")?;
            }
            Self::UInt32(n) => {
                write_indent(indent, w)?;
                writeln!(w, "u32: {n}")?;
            }
            Self::Int64(n) => {
                write_indent(indent, w)?;
                writeln!(w, "i64: {n}")?;
            }
            Self::UInt64(n) => {
                write_indent(indent, w)?;
                writeln!(w, "u64: {n}")?;
            }
            Self::Double(n) => {
                write_indent(indent, w)?;
                writeln!(w, "double: {n}")?;
            }
            Self::UnixFD(n) => {
                write_indent(indent, w)?;
                writeln!(w, "unixfd: {n}")?;
            }
            Self::String(s) => {
                write_indent(indent, w)?;
                writeln!(w, "string: {s:?}")?;
            }
            Self::ObjectPath(path) => {
                write_indent(indent, w)?;
                writeln!(w, "path: {path:?}")?;
            }
            Self::Signature(signature) => {
                write_indent(indent, w)?;
                writeln!(w, "signature: {signature:?}")?;
            }
            Self::Struct(struct_) => {
                let mut iter = struct_.fields_iter().map_err(|_| core::fmt::Error)?;
                write_indent(indent, w)?;
                writeln!(w, "struct:")?;
                while let Some(item) = iter.try_next().map_err(|_| core::fmt::Error)? {
                    item.log(w, indent.checked_add(4).ok_or(core::fmt::Error)?)?;
                }
            }
            Self::Array(array) => {
                let mut iter = array.items_iter();
                write_indent(indent, w)?;
                writeln!(w, "array:")?;
                while let Some(item) = iter.try_next().map_err(|_| core::fmt::Error)? {
                    item.log(w, indent.checked_add(4).ok_or(core::fmt::Error)?)?;
                }
            }
            Self::DictEntry(pair) => {
                let (key, value) = pair.key_value().map_err(|_| core::fmt::Error)?;

                write_indent(indent, w)?;
                writeln!(w, "dict:")?;

                write_indent(indent.checked_add(4).ok_or(core::fmt::Error)?, w)?;
                writeln!(w, "key:")?;
                key.log(w, indent.checked_add(8).ok_or(core::fmt::Error)?)?;

                write_indent(indent.checked_add(4).ok_or(core::fmt::Error)?, w)?;
                writeln!(w, "value:")?;
                value.log(w, indent.checked_add(8).ok_or(core::fmt::Error)?)?;
            }
            Self::Variant(variant) => {
                write_indent(indent, w)?;
                writeln!(w, "variant:")?;

                let value = variant.materialize().map_err(|_| core::fmt::Error)?;
                value.log(w, indent.checked_add(4).ok_or(core::fmt::Error)?)?;
            }
        }

        Ok(())
    }
}
