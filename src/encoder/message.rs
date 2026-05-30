use crate::{
    MessageType,
    const_helpers::{t_err, u8_from_usize, u32_from_usize},
    encoder::{EncodeError, EncodeResult, cursor::SliceCursor},
    types::HeaderFieldCode,
};

/// A zero-allocation D-Bus message encoder that writes into a caller-provided byte slice.
#[derive(Debug)]
pub struct MessageEncoder<'buf> {
    cur: SliceCursor<'buf>,
    header_fields_start: usize,
    body_start: Option<usize>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct __DbusArrayFrame {
    len_pos: usize,
    data_start: usize,
}

impl<'buf> MessageEncoder<'buf> {
    /// Creates a new encoder over `buf`.
    ///
    /// # Errors
    ///
    /// Returns an error if there's not enough space in the given `buf`.
    pub const fn new(
        buf: &'buf mut [u8],
        message_type: MessageType,
        serial: u32,
    ) -> EncodeResult<Self> {
        let mut cur = SliceCursor::new(buf);
        t_err!(cur.write_u8(b'l'));
        t_err!(cur.write_u8(message_type.into_u8()));
        t_err!(cur.write_u8(0));
        t_err!(cur.write_u8(1));
        t_err!(cur.write_u32(0));
        t_err!(cur.write_u32(serial));
        t_err!(cur.write_u32(0));
        Ok(Self {
            cur,
            header_fields_start: 16,
            body_start: None,
        })
    }

    /// Sets the object path header field.
    ///
    /// # Errors
    ///
    /// Returns an error if the header is already closed, the output buffer is too small, or the
    /// encoded message would exceed a D-Bus length limit.
    pub const fn set_path(&mut self, path: &str) -> EncodeResult<()> {
        t_err!(self.ensure_header_open());
        write_header_string(&mut self.cur, HeaderFieldCode::Path, b'o', path)
    }

    /// Sets the interface header field.
    ///
    /// # Errors
    ///
    /// Returns an error if the header is already closed, the output buffer is too small, or the
    /// encoded message would exceed a D-Bus length limit.
    pub const fn set_interface(&mut self, interface: &str) -> EncodeResult<()> {
        t_err!(self.ensure_header_open());
        write_header_string(&mut self.cur, HeaderFieldCode::Interface, b's', interface)
    }

    /// Sets the member header field.
    ///
    /// # Errors
    ///
    /// Returns an error if the header is already closed, the output buffer is too small, or the
    /// encoded message would exceed a D-Bus length limit.
    pub const fn set_member(&mut self, member: &str) -> EncodeResult<()> {
        t_err!(self.ensure_header_open());
        write_header_string(&mut self.cur, HeaderFieldCode::Member, b's', member)
    }

    /// Sets the error name header field.
    ///
    /// # Errors
    ///
    /// Returns an error if the header is already closed, the output buffer is too small, or the
    /// encoded message would exceed a D-Bus length limit.
    pub const fn set_error_name(&mut self, error_name: &str) -> EncodeResult<()> {
        t_err!(self.ensure_header_open());
        write_header_string(&mut self.cur, HeaderFieldCode::ErrorName, b's', error_name)
    }

    /// Sets the reply serial header field.
    ///
    /// # Errors
    ///
    /// Returns an error if the header is already closed, the output buffer is too small, or the
    /// encoded message would exceed a D-Bus length limit.
    pub const fn set_reply_serial(&mut self, reply_serial: u32) -> EncodeResult<()> {
        t_err!(self.ensure_header_open());
        write_header_u32(&mut self.cur, HeaderFieldCode::ReplySerial, reply_serial)
    }

    /// Sets the destination header field.
    ///
    /// # Errors
    ///
    /// Returns an error if the header is already closed, the output buffer is too small, or the
    /// encoded message would exceed a D-Bus length limit.
    pub const fn set_destination(&mut self, destination: &str) -> EncodeResult<()> {
        t_err!(self.ensure_header_open());
        write_header_string(
            &mut self.cur,
            HeaderFieldCode::Destination,
            b's',
            destination,
        )
    }

    /// Sets the sender header field.
    ///
    /// # Errors
    ///
    /// Returns an error if the header is already closed, the output buffer is too small, or the
    /// encoded message would exceed a D-Bus length limit.
    pub const fn set_sender(&mut self, sender: &str) -> EncodeResult<()> {
        t_err!(self.ensure_header_open());
        write_header_string(&mut self.cur, HeaderFieldCode::Sender, b's', sender)
    }

    /// Sets the unix file descriptor count header field.
    ///
    /// # Errors
    ///
    /// Returns an error if the header is already closed, the output buffer is too small, or the
    /// encoded message would exceed a D-Bus length limit.
    pub const fn set_unix_fds(&mut self, unix_fds: u32) -> EncodeResult<()> {
        t_err!(self.ensure_header_open());
        write_header_u32(&mut self.cur, HeaderFieldCode::UnixFds, unix_fds)
    }

    /// Sets the body signature header field from a caller-provided D-Bus signature.
    ///
    /// # Errors
    ///
    /// Returns an error if the header is already closed, the output buffer is too small, or the
    /// encoded message would exceed a D-Bus length limit.
    pub const fn set_body_signature(&mut self, signature: &str) -> EncodeResult<()> {
        t_err!(self.ensure_header_open());
        if signature.is_empty() {
            return Ok(());
        }
        t_err!(self.cur.align(8));
        t_err!(self.cur.write_u8(HeaderFieldCode::Signature.into_u8()));
        t_err!(write_variant_signature(&mut self.cur, b'g'));
        t_err!(write_signature(&mut self.cur, signature));
        Ok(())
    }

    #[doc(hidden)]
    pub const fn __dbus_begin_body(&mut self) -> EncodeResult<()> {
        self.ensure_body_started()
    }

    #[doc(hidden)]
    pub const fn __dbus_align(&mut self, align: usize) -> EncodeResult<()> {
        self.cur.align(align)
    }

    #[doc(hidden)]
    pub const fn __dbus_write_u8(&mut self, value: u8) -> EncodeResult<()> {
        self.cur.write_u8(value)
    }

    #[doc(hidden)]
    pub const fn __dbus_write_bool(&mut self, value: bool) -> EncodeResult<()> {
        t_err!(self.cur.align(4));
        self.cur.write_u32(value as u32)
    }

    #[doc(hidden)]
    pub const fn __dbus_write_i16(&mut self, value: i16) -> EncodeResult<()> {
        t_err!(self.cur.align(2));
        self.cur.write_i16(value)
    }

    #[doc(hidden)]
    pub const fn __dbus_write_u16(&mut self, value: u16) -> EncodeResult<()> {
        t_err!(self.cur.align(2));
        self.cur.write_u16(value)
    }

    #[doc(hidden)]
    pub const fn __dbus_write_i32(&mut self, value: i32) -> EncodeResult<()> {
        t_err!(self.cur.align(4));
        self.cur.write_i32(value)
    }

    #[doc(hidden)]
    pub const fn __dbus_write_u32(&mut self, value: u32) -> EncodeResult<()> {
        t_err!(self.cur.align(4));
        self.cur.write_u32(value)
    }

    #[doc(hidden)]
    pub const fn __dbus_write_i64(&mut self, value: i64) -> EncodeResult<()> {
        t_err!(self.cur.align(8));
        self.cur.write_i64(value)
    }

    #[doc(hidden)]
    pub const fn __dbus_write_u64(&mut self, value: u64) -> EncodeResult<()> {
        t_err!(self.cur.align(8));
        self.cur.write_u64(value)
    }

    #[doc(hidden)]
    pub const fn __dbus_write_f64(&mut self, value: f64) -> EncodeResult<()> {
        t_err!(self.cur.align(8));
        self.cur.write_f64(value)
    }

    #[doc(hidden)]
    pub const fn __dbus_write_unix_fd(&mut self, value: u32) -> EncodeResult<()> {
        t_err!(self.cur.align(4));
        self.cur.write_u32(value)
    }

    #[doc(hidden)]
    pub const fn __dbus_write_string_like(&mut self, value: &str) -> EncodeResult<()> {
        write_string_like(&mut self.cur, value)
    }

    #[doc(hidden)]
    pub const fn __dbus_write_signature_value(&mut self, value: &str) -> EncodeResult<()> {
        write_signature(&mut self.cur, value)
    }

    #[doc(hidden)]
    pub const fn __dbus_start_array(
        &mut self,
        item_alignment: usize,
    ) -> EncodeResult<__DbusArrayFrame> {
        t_err!(self.cur.align(4));
        let len_pos = self.cur.pos();
        t_err!(self.cur.write_u32(0));
        t_err!(self.cur.align(item_alignment));
        let data_start = self.cur.pos();
        Ok(__DbusArrayFrame {
            len_pos,
            data_start,
        })
    }

    #[doc(hidden)]
    pub const fn __dbus_finish_array(&mut self, frame: __DbusArrayFrame) -> EncodeResult<()> {
        let Some(byte_len) = self.cur.pos().checked_sub(frame.data_start) else {
            return Err(EncodeError::ContainerTooLong);
        };
        let Some(byte_len) = u32_from_usize(byte_len) else {
            return Err(EncodeError::ContainerTooLong);
        };
        self.cur.set_u32(frame.len_pos, byte_len)
    }

    /// Finishes the message and returns the encoded byte length.
    ///
    /// # Errors
    ///
    /// Returns an error if the header or body length cannot be finalized.
    pub const fn finish(mut self) -> EncodeResult<usize> {
        t_err!(self.ensure_body_started());
        let Some(body_start) = self.body_start else {
            return Err(EncodeError::BodySignatureIncomplete);
        };

        let Some(body_len) = self.cur.pos().checked_sub(body_start) else {
            return Err(EncodeError::ContainerTooLong);
        };

        let Some(body_len) = u32_from_usize(body_len) else {
            return Err(EncodeError::ContainerTooLong);
        };
        t_err!(self.cur.set_u32(4, body_len));
        Ok(self.cur.pos())
    }

    const fn ensure_header_open(&self) -> EncodeResult<()> {
        if self.body_start.is_some() {
            return Err(EncodeError::HeaderAlreadyFinished);
        }
        Ok(())
    }

    const fn ensure_body_started(&mut self) -> EncodeResult<()> {
        if self.body_start.is_none() {
            let Some(header_fields_len) = self.cur.pos().checked_sub(self.header_fields_start)
            else {
                return Err(EncodeError::ContainerTooLong);
            };
            let Some(header_fields_len) = u32_from_usize(header_fields_len) else {
                return Err(EncodeError::ContainerTooLong);
            };
            t_err!(self.cur.set_u32(12, header_fields_len));
            t_err!(self.cur.align(8));
            self.body_start = Some(self.cur.pos());
        }
        Ok(())
    }
}

const fn write_header_string(
    cur: &mut SliceCursor<'_>,
    field: HeaderFieldCode,
    value_signature: u8,
    value: &str,
) -> EncodeResult<()> {
    t_err!(cur.align(8));
    t_err!(cur.write_u8(field.into_u8()));
    t_err!(write_variant_signature(cur, value_signature));
    t_err!(write_string_like(cur, value));
    Ok(())
}

const fn write_header_u32(
    cur: &mut SliceCursor<'_>,
    field: HeaderFieldCode,
    value: u32,
) -> EncodeResult<()> {
    t_err!(cur.align(8));
    t_err!(cur.write_u8(field.into_u8()));
    t_err!(write_variant_signature(cur, b'u'));
    t_err!(cur.align(4));
    t_err!(cur.write_u32(value));
    Ok(())
}

const fn write_variant_signature(cur: &mut SliceCursor<'_>, signature: u8) -> EncodeResult<()> {
    t_err!(cur.write_u8(1));
    t_err!(cur.write_u8(signature));
    t_err!(cur.write_u8(0));
    Ok(())
}

const fn write_string_like(cur: &mut SliceCursor<'_>, value: &str) -> EncodeResult<()> {
    let Some(len) = u32_from_usize(value.len()) else {
        return Err(EncodeError::ValueTooLong);
    };

    t_err!(cur.align(4));
    t_err!(cur.write_u32(len));
    t_err!(cur.write_bytes(value.as_bytes()));
    t_err!(cur.write_u8(0));
    Ok(())
}

const fn write_signature(cur: &mut SliceCursor<'_>, value: &str) -> EncodeResult<()> {
    let Some(len) = u8_from_usize(value.len()) else {
        return Err(EncodeError::ValueTooLong);
    };

    t_err!(cur.write_u8(len));
    t_err!(cur.write_bytes(value.as_bytes()));
    t_err!(cur.write_u8(0));
    Ok(())
}
