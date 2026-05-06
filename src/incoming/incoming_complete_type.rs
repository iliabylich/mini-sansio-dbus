use crate::{DBusError, incoming::Cursor};

#[derive(Debug, Clone, Copy)]
#[must_use]
pub(crate) enum IncomingCompleteType<'a> {
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
    Struct {
        full: &'a str,
    },
    Array {
        full: &'a str,
        item: &'a str,
    },
    DictEntry {
        full: &'a str,
        key: &'a str,
        value: &'a str,
    },
    Variant,
}
impl<'a> IncomingCompleteType<'a> {
    pub(crate) fn cut(buf: &'a str) -> Result<(Self, &'a str), DBusError> {
        let (start, mut remainder) = buf
            .split_at_checked(1)
            .ok_or(DBusError::MalformedSignature)?;
        let start = start.chars().next().ok_or(DBusError::MalformedSignature)?;

        if start == '(' {
            while !remainder.starts_with(')') {
                let (_, leftover) = Self::cut(remainder)?;
                remainder = leftover;
            }
            remainder = remainder
                .strip_prefix(')')
                .ok_or(DBusError::MalformedSignature)?;
            let len = buf.len() - remainder.len();
            let full = &buf[..len];
            return Ok((Self::Struct { full }, remainder));
        } else if start == '{' {
            let (key, leftover) = Self::cut(remainder)?;
            remainder = leftover;

            let (value, leftover) = Self::cut(remainder)?;
            remainder = leftover;

            remainder = remainder
                .strip_prefix('}')
                .ok_or(DBusError::MalformedSignature)?;
            let len = buf.len() - remainder.len();
            let full = &buf[..len];

            return Ok((
                Self::DictEntry {
                    full,
                    key: key.buf(),
                    value: value.buf(),
                },
                remainder,
            ));
        } else if start == 'a' {
            let (item, remainder) = Self::cut(remainder)?;
            let full = &buf[..=item.buf().len()];
            return Ok((
                Self::Array {
                    full,
                    item: item.buf(),
                },
                remainder,
            ));
        }

        let simple = match start {
            'y' => Self::Byte,
            'b' => Self::Bool,
            'n' => Self::Int16,
            'q' => Self::UInt16,
            'i' => Self::Int32,
            'u' => Self::UInt32,
            'x' => Self::Int64,
            't' => Self::UInt64,
            'd' => Self::Double,
            'h' => Self::UnixFD,
            's' => Self::String,
            'o' => Self::ObjectPath,
            'g' => Self::Signature,
            'v' => Self::Variant,

            _ => return Err(DBusError::MalformedSignature),
        };
        Ok((simple, remainder))
    }

    pub(crate) const fn buf(&self) -> &'a str {
        match self {
            Self::Byte => "y",
            Self::Bool => "b",
            Self::Int16 => "n",
            Self::UInt16 => "q",
            Self::Int32 => "i",
            Self::UInt32 => "u",
            Self::Int64 => "x",
            Self::UInt64 => "t",
            Self::Double => "d",
            Self::UnixFD => "h",
            Self::String => "s",
            Self::ObjectPath => "o",
            Self::Signature => "g",
            Self::Struct { full } | Self::Array { full, .. } | Self::DictEntry { full, .. } => full,
            Self::Variant => "v",
        }
    }

    pub(crate) fn bytesize(&self, mut cur: Cursor<'_>) -> Result<usize, DBusError> {
        let bytesize = match self {
            Self::Byte => 1,

            Self::Int16 | Self::UInt16 => 2,
            Self::Bool | Self::Int32 | Self::UInt32 | Self::UnixFD => 4,

            Self::Int64 | Self::UInt64 | Self::Double => 8,

            Self::String | Self::ObjectPath => {
                let len = cur.cut_u32().map_err(|_| DBusError::MalformedSignature)?;
                4 + len as usize + 1
            }
            Self::Signature => {
                let len = cur.cut_u8().map_err(|_| DBusError::MalformedSignature)?;
                1 + len as usize + 1
            }
            Self::Struct { full } => {
                let mut len = 0;
                let mut iter = CompleteTypeStructFieldsIter::new(full)?;
                while let Some(field_type) = iter.try_next()? {
                    let old_offset = cur.offset();
                    cur.align(field_type.alignment())?;
                    len += cur.offset() - old_offset;

                    let fieldsize = field_type.bytesize(cur)?;
                    len += fieldsize;

                    cur.take(fieldsize)?;
                }
                len
            }
            Self::Array { item, .. } => {
                let bytesize = cur.cut_u32()?;
                let (item_type, leftover) = Self::cut(item)?;
                if !leftover.is_empty() {
                    return Err(DBusError::MalformedSignature);
                }
                let old_offset = cur.offset();
                cur.align(item_type.alignment())?;
                let offset = cur.offset() - old_offset;
                4 + offset + bytesize as usize
            }
            Self::DictEntry { key, value, .. } => {
                let (key_type, leftover) = Self::cut(key)?;
                if !leftover.is_empty() {
                    return Err(DBusError::MalformedSignature);
                }

                let (value_type, leftover) = Self::cut(value)?;
                if !leftover.is_empty() {
                    return Err(DBusError::MalformedSignature);
                }

                let old_offset = cur.offset();
                cur.align(key_type.alignment())?;
                let keyoffset = cur.offset() - old_offset;
                let keysize = key_type.bytesize(cur)?;
                cur.take(keysize)?;

                let old_offset = cur.offset();
                cur.align(value_type.alignment())?;
                let valueoffset = cur.offset() - old_offset;
                let valuesize = value_type.bytesize(cur)?;

                keyoffset + keysize + valueoffset + valuesize
            }
            Self::Variant => {
                let signature = cur.cut_signature()?;
                let (complete_type, leftover) = IncomingCompleteType::cut(signature)?;
                if !leftover.is_empty() {
                    return Err(DBusError::MalformedSignature);
                }

                let old_offset = cur.offset();
                cur.align(complete_type.alignment())?;
                let offset = cur.offset() - old_offset;
                let valuesize = complete_type.bytesize(cur)?;

                1 + signature.len() + 1 + offset + valuesize
            }
        };

        Ok(bytesize)
    }

    pub(crate) const fn alignment(&self) -> usize {
        match self {
            Self::Byte | Self::Variant | Self::Signature => 1,

            Self::Int16 | Self::UInt16 => 2,

            Self::UnixFD
            | Self::String
            | Self::ObjectPath
            | Self::Bool
            | Self::Int32
            | Self::UInt32
            | Self::Array { .. } => 4,

            Self::Int64
            | Self::UInt64
            | Self::Double
            | Self::Struct { .. }
            | Self::DictEntry { .. } => 8,
        }
    }
}

pub(crate) struct CompleteTypeStructFieldsIter<'a> {
    fields: &'a str,
}
impl<'a> CompleteTypeStructFieldsIter<'a> {
    pub(crate) fn new(full: &'a str) -> Result<Self, DBusError> {
        Ok(Self {
            fields: full
                .strip_prefix('(')
                .ok_or(DBusError::MalformedSignature)?
                .strip_suffix(')')
                .ok_or(DBusError::MalformedSignature)?,
        })
    }

    pub(crate) fn try_next(&mut self) -> Result<Option<IncomingCompleteType<'a>>, DBusError> {
        if self.fields.is_empty() {
            return Ok(None);
        }

        let (out, remainder) = IncomingCompleteType::cut(self.fields)?;
        self.fields = remainder;
        Ok(Some(out))
    }
}
