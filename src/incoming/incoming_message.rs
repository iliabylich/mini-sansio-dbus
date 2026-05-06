use crate::{
    DBusError, IncomingBody, MessageType,
    incoming::{Cursor, HeaderFields},
    types::Header,
};

/// Received message
#[derive(Clone, Copy)]
#[must_use]
#[expect(missing_docs)]
pub struct IncomingMessage<'a> {
    pub message_type: MessageType,
    pub serial: u32,

    pub path: Option<&'a str>,
    pub interface: Option<&'a str>,
    pub member: Option<&'a str>,
    pub error_name: Option<&'a str>,
    pub reply_serial: Option<u32>,
    pub destination: Option<&'a str>,
    pub sender: Option<&'a str>,
    pub signature: Option<&'a str>,
    pub unix_fds: Option<u32>,

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
            let body_padding = (8 - (header_fields_len as usize % 8)) % 8;
            let body_buf = cur
                .buf()
                .get(body_padding..body_padding + body_len as usize)
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
    pub fn log(&self) -> Result<(), DBusError> {
        eprintln!("============");
        eprintln!("Type = {:?}", self.message_type);
        eprintln!("Serial = {}", self.serial);
        eprintln!("Path = {:?}", self.path);
        eprintln!("Interface = {:?}", self.interface);
        eprintln!("Member = {:?}", self.member);
        eprintln!("ErrorName = {:?}", self.error_name);
        eprintln!("ReplySerial = {:?}", self.reply_serial);
        eprintln!("Destination = {:?}", self.destination);
        eprintln!("Sender = {:?}", self.sender);
        eprintln!("Signature = {:?}", self.signature);
        eprintln!("UnixFDs = {:?}", self.unix_fds);

        if let Some(mut body) = self.body {
            eprintln!("Body:");
            while let Some(value) = body.try_next()? {
                value.log(4)?;
            }
        }
        eprintln!("============");

        Ok(())
    }
}
