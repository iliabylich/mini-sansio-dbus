use crate::{
    DBusError, IncomingValue,
    incoming::{Cursor, IncomingCompleteType},
};

/// Represents a received dict entry (a key/value pair)
#[derive(Debug)]
#[must_use]
pub struct IncomingDictEntryValue<'a> {
    key_type: IncomingCompleteType<'a>,
    value_type: IncomingCompleteType<'a>,
    cur: Cursor<'a>,
}

impl<'a> IncomingDictEntryValue<'a> {
    pub(crate) fn cut(
        cur: &Cursor<'a>,
        key_sig: &'a str,
        value_sig: &'a str,
    ) -> Result<Self, DBusError> {
        let (key_type, leftover) = IncomingCompleteType::cut(key_sig)?;
        if !leftover.is_empty() {
            return Err(DBusError::MalformedDictEntry);
        }

        let (value_type, leftover) = IncomingCompleteType::cut(value_sig)?;
        if !leftover.is_empty() {
            return Err(DBusError::MalformedDictEntry);
        }

        Ok(Self {
            key_type,
            value_type,
            cur: *cur,
        })
    }

    /// Returns key and value of `self`
    ///
    /// # Errors
    ///
    /// Returns an error if any lazily parsed value inside `self` is invalid
    pub fn key_value(&self) -> Result<(IncomingValue<'a>, IncomingValue<'a>), DBusError> {
        let mut cur = self.cur;
        let key = IncomingValue::cut(&mut cur, self.key_type)?;
        let value = IncomingValue::cut(&mut cur, self.value_type)?;
        if !cur.buf().is_empty() {
            return Err(DBusError::MalformedDictEntry);
        }
        Ok((key, value))
    }
}
