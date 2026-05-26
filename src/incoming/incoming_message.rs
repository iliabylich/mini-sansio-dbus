use crate::{
    DBusError, IncomingBody, MessageType,
    incoming::{Cursor, HeaderFields},
    types::Header,
};

/// Received message
#[derive(Clone, Copy)]
#[must_use]
pub struct IncomingMessage<'a> {
    /// Type
    pub message_type: MessageType,
    /// Serial
    pub serial: u32,

    /// Path
    pub path: Option<&'a str>,
    /// Interface
    pub interface: Option<&'a str>,
    /// Member
    pub member: Option<&'a str>,
    /// `ErrorName`
    pub error_name: Option<&'a str>,
    /// `ReplySerial`
    pub reply_serial: Option<u32>,
    /// Destination
    pub destination: Option<&'a str>,
    /// Sender
    pub sender: Option<&'a str>,
    /// Signature
    pub signature: Option<&'a str>,
    /// `UnixFDs`
    pub unix_fds: Option<u32>,

    /// Message body
    pub body: Option<IncomingBody<'a>>,
}

impl<'a> IncomingMessage<'a> {
    pub(crate) fn new(buf: &'a [u8]) -> Result<Self, DBusError> {
        let mut cur = Cursor::new(buf, 0);

        let Header {
            _endian,
            message_type,
            _flags,
            _protocol_version,
            body_len,
            serial,
            header_fields_len,
        } = Header::cut(&mut cur)?;
        let message_type = MessageType::try_from(message_type)?;

        let headers = cur.take(header_fields_len as usize)?;

        let HeaderFields {
            path,
            interface,
            member,
            error_name,
            reply_serial,
            destination,
            sender,
            signature,
            unix_fds,
        } = HeaderFields::cut(Cursor::new(headers, 0))?;

        let body = if let Some(signature) = signature {
            let padded_header_len = (header_fields_len as usize)
                .checked_next_multiple_of(8)
                .ok_or(DBusError::MalformedBody)?;
            let body_padding = padded_header_len
                .checked_sub(header_fields_len as usize)
                .ok_or(DBusError::MalformedBody)?;
            let body_end = body_padding
                .checked_add(body_len as usize)
                .ok_or(DBusError::MalformedBody)?;
            let body_buf = cur
                .buf()
                .get(body_padding..body_end)
                .ok_or(DBusError::MalformedBody)?;
            Some(IncomingBody::new(signature, Cursor::new(body_buf, 0)))
        } else {
            None
        };

        Ok(Self {
            message_type,
            serial,

            path,
            interface,
            member,
            error_name,
            reply_serial,
            destination,
            sender,
            signature,
            unix_fds,

            body,
        })
    }

    /// Prints `self` to stderr
    ///
    /// # Errors
    ///
    /// Returns an error if any lazily parsed value inside `self` are invalid
    pub fn log(&self, w: &mut impl core::fmt::Write) -> Result<(), core::fmt::Error> {
        writeln!(w, "============")?;
        writeln!(w, "Type = {:?}", self.message_type)?;
        writeln!(w, "Serial = {}", self.serial)?;
        writeln!(w, "Path = {:?}", self.path)?;
        writeln!(w, "Interface = {:?}", self.interface)?;
        writeln!(w, "Member = {:?}", self.member)?;
        writeln!(w, "ErrorName = {:?}", self.error_name)?;
        writeln!(w, "ReplySerial = {:?}", self.reply_serial)?;
        writeln!(w, "Destination = {:?}", self.destination)?;
        writeln!(w, "Sender = {:?}", self.sender)?;
        writeln!(w, "Signature = {:?}", self.signature)?;
        writeln!(w, "UnixFDs = {:?}", self.unix_fds)?;

        if let Some(mut body) = self.body {
            writeln!(w, "Body:")?;
            while let Some(value) = body.try_next().map_err(|_| core::fmt::Error)? {
                value.log(w, 4)?;
            }
        }
        writeln!(w, "============")?;

        Ok(())
    }
}
