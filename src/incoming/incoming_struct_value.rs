use crate::{
    DBusError, IncomingValue,
    incoming::{CompleteTypeStructFieldsIter, Cursor, IncomingCompleteType},
};

/// Received struct (an array of dict entries)
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct IncomingStructValue<'a> {
    struct_type: IncomingCompleteType<'a>,
    cur: Cursor<'a>,
}

impl<'a> IncomingStructValue<'a> {
    pub(crate) const fn new(struct_type: IncomingCompleteType<'a>, cur: Cursor<'a>) -> Self {
        Self { struct_type, cur }
    }

    /// Returns an iterator over struct fields
    ///
    /// # Errors
    ///
    /// Returns an error if lazily parsed struct signature is invalid
    pub fn fields_iter(&self) -> Result<IncomingStructValueIter<'a>, DBusError> {
        Ok(IncomingStructValueIter {
            field_type_iter: CompleteTypeStructFieldsIter::new(self.struct_type.buf())?,
            cur: self.cur,
        })
    }
}

/// An iterator over `IncomingStructValue`
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct IncomingStructValueIter<'a> {
    field_type_iter: CompleteTypeStructFieldsIter<'a>,
    cur: Cursor<'a>,
}

impl<'a> IncomingStructValueIter<'a> {
    /// Takes the next key/value tuple out of `self`
    ///
    /// # Errors
    ///
    /// Returns an error if the next value is invalid
    pub fn try_next(&mut self) -> Result<Option<IncomingValue<'a>>, DBusError> {
        let Some(field_type) = self.field_type_iter.try_next()? else {
            return Ok(None);
        };

        let value = IncomingValue::cut(&mut self.cur, field_type)?;

        Ok(Some(value))
    }
}
