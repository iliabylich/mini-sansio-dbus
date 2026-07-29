use crate::{
    DBusError, IncomingValue,
    incoming::{Cursor, IncomingCompleteType},
};

/// Represents a variant value known only at runtime
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct IncomingVariantValue<'a> {
    cur: Cursor<'a>,
}

impl<'a> IncomingVariantValue<'a> {
    pub(crate) const fn new(cur: Cursor<'a>) -> Self {
        Self { cur }
    }

    /// Constructs an `IncomingValue` objects out of lazy blob of bytes stored in `self`
    ///
    /// # Errors
    ///
    /// Returns an error if contains invalid signature or value bytes don't match the signature
    pub fn materialize(&self) -> Result<IncomingValue<'a>, DBusError> {
        let mut cur = self.cur;
        let signature = cur.cut_signature()?;

        let (type_, leftover) = IncomingCompleteType::cut(signature)?;
        if !leftover.is_empty() {
            return Err(DBusError::MalformedVariant);
        }

        let value = IncomingValue::cut(&mut cur, type_)?;
        if !cur.buf().is_empty() {
            return Err(DBusError::MalformedVariant);
        }

        Ok(value)
    }
}
