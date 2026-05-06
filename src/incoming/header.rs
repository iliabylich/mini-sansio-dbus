use crate::{DBusError, incoming::Cursor, types::Header};

impl Header {
    pub(crate) fn cut(cur: &mut Cursor<'_>) -> Result<Self, DBusError> {
        let bytes = cur
            .take(size_of::<Self>())
            .map_err(|_| DBusError::NoHeader)?;
        Self::from_bytes(bytes)
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, DBusError> {
        let mut cursor = Cursor::new(bytes, 0);
        let endian = cursor.cut_u8()?;
        let message_type = cursor.cut_u8()?;
        let flags = cursor.cut_u8()?;
        let protocol_version = cursor.cut_u8()?;
        let body_len = cursor.cut_u32()?;
        let serial = cursor.cut_u32()?;
        let header_fields_len = cursor.cut_u32()?;
        Ok(Self {
            _endian: endian,
            message_type,
            _flags: flags,
            _protocol_version: protocol_version,
            body_len,
            serial,
            header_fields_len,
        })
    }
}
