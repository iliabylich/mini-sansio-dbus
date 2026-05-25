use crate::{
    MessageType,
    encoder::{
        EncodeError, EncodeResult,
        cursor::SliceCursor,
        types::{DbusType, write_signature, write_string_like},
    },
    types::HeaderFieldCode,
};

/// A zero-allocation D-Bus message encoder that writes into a caller-provided byte slice.
#[derive(Debug)]
pub struct MessageEncoder<'buf> {
    cur: SliceCursor<'buf>,
    header_fields_start: usize,
    body_start: Option<usize>,
}

impl<'buf> MessageEncoder<'buf> {
    /// Creates a new encoder over `buf`.
    pub fn new(buf: &'buf mut [u8], message_type: MessageType, serial: u32) -> EncodeResult<Self> {
        let mut cur = SliceCursor::new(buf);
        cur.write_u8(b'l')?;
        cur.write_u8(message_type.into())?;
        cur.write_u8(0)?;
        cur.write_u8(1)?;
        cur.write_u32(0)?;
        cur.write_u32(serial)?;
        cur.write_u32(0)?;
        Ok(Self {
            cur,
            header_fields_start: 16,
            body_start: None,
        })
    }

    /// Sets the object path header field.
    pub fn set_path(&mut self, path: &str) -> EncodeResult<()> {
        self.ensure_header_open()?;
        write_header_string(&mut self.cur, HeaderFieldCode::Path, b'o', path)
    }

    /// Sets the interface header field.
    pub fn set_interface(&mut self, interface: &str) -> EncodeResult<()> {
        self.ensure_header_open()?;
        write_header_string(&mut self.cur, HeaderFieldCode::Interface, b's', interface)
    }

    /// Sets the member header field.
    pub fn set_member(&mut self, member: &str) -> EncodeResult<()> {
        self.ensure_header_open()?;
        write_header_string(&mut self.cur, HeaderFieldCode::Member, b's', member)
    }

    /// Sets the error name header field.
    pub fn set_error_name(&mut self, error_name: &str) -> EncodeResult<()> {
        self.ensure_header_open()?;
        write_header_string(&mut self.cur, HeaderFieldCode::ErrorName, b's', error_name)
    }

    /// Sets the reply serial header field.
    pub fn set_reply_serial(&mut self, reply_serial: u32) -> EncodeResult<()> {
        self.ensure_header_open()?;
        write_header_u32(&mut self.cur, HeaderFieldCode::ReplySerial, reply_serial)
    }

    /// Sets the destination header field.
    pub fn set_destination(&mut self, destination: &str) -> EncodeResult<()> {
        self.ensure_header_open()?;
        write_header_string(
            &mut self.cur,
            HeaderFieldCode::Destination,
            b's',
            destination,
        )
    }

    /// Sets the sender header field.
    pub fn set_sender(&mut self, sender: &str) -> EncodeResult<()> {
        self.ensure_header_open()?;
        write_header_string(&mut self.cur, HeaderFieldCode::Sender, b's', sender)
    }

    /// Sets the unix file descriptor count header field.
    pub fn set_unix_fds(&mut self, unix_fds: u32) -> EncodeResult<()> {
        self.ensure_header_open()?;
        write_header_u32(&mut self.cur, HeaderFieldCode::UnixFds, unix_fds)
    }

    /// Sets the body signature header field from a caller-provided D-Bus signature.
    pub fn set_body_signature(&mut self, signature: &str) -> EncodeResult<()> {
        self.ensure_header_open()?;
        if signature.is_empty() {
            return Ok(());
        }
        self.cur.align(8)?;
        self.cur.write_u8(HeaderFieldCode::Signature.into())?;
        write_variant_signature(&mut self.cur, b'g')?;
        write_signature(&mut self.cur, signature)
    }

    /// Starts the next top-level body slot.
    pub fn next_body_slot<T: DbusType>(&mut self) -> EncodeResult<T::Slot<'_, 'buf>> {
        self.ensure_body_started()?;
        T::start_slot(&mut self.cur)
    }

    /// Finishes the message and returns the encoded byte length.
    pub fn finish(mut self) -> EncodeResult<usize> {
        self.ensure_body_started()?;
        let body_start = self
            .body_start
            .ok_or(EncodeError::BodySignatureIncomplete)?;
        let body_len = self
            .cur
            .pos()
            .checked_sub(body_start)
            .ok_or(EncodeError::ContainerTooLong)?;
        let body_len = u32::try_from(body_len).map_err(|_| EncodeError::ContainerTooLong)?;
        self.cur.set_u32(4, body_len)?;
        Ok(self.cur.pos())
    }

    fn ensure_header_open(&self) -> EncodeResult<()> {
        if self.body_start.is_some() {
            return Err(EncodeError::HeaderAlreadyFinished);
        }
        Ok(())
    }

    fn ensure_body_started(&mut self) -> EncodeResult<()> {
        if self.body_start.is_none() {
            let header_fields_len = self
                .cur
                .pos()
                .checked_sub(self.header_fields_start)
                .ok_or(EncodeError::ContainerTooLong)?;
            let header_fields_len =
                u32::try_from(header_fields_len).map_err(|_| EncodeError::ContainerTooLong)?;
            self.cur.set_u32(12, header_fields_len)?;
            self.cur.align(8)?;
            self.body_start = Some(self.cur.pos());
        }
        Ok(())
    }
}

fn write_header_string(
    cur: &mut SliceCursor<'_>,
    field: HeaderFieldCode,
    value_signature: u8,
    value: &str,
) -> EncodeResult<()> {
    cur.align(8)?;
    cur.write_u8(field.into())?;
    write_variant_signature(cur, value_signature)?;
    write_string_like(cur, value)
}

fn write_header_u32(
    cur: &mut SliceCursor<'_>,
    field: HeaderFieldCode,
    value: u32,
) -> EncodeResult<()> {
    cur.align(8)?;
    cur.write_u8(field.into())?;
    write_variant_signature(cur, b'u')?;
    cur.align(4)?;
    cur.write_u32(value)
}

fn write_variant_signature(cur: &mut SliceCursor<'_>, signature: u8) -> EncodeResult<()> {
    cur.write_u8(1)?;
    cur.write_u8(signature)?;
    cur.write_u8(0)
}
