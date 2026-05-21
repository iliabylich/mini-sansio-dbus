use crate::{DBusError, DBusWants, types::Header};

const HEADER_LEN: usize = size_of::<Header>();

pub(crate) struct DBusReader {
    state: State,
    seq: u64,
}

#[derive(Debug, Clone, Copy)]
enum State {
    ReadHeader {
        bytes_read: usize,
    },
    ReadBody {
        message_len: usize,
        bytes_read: usize,
    },
    Dead,
}

impl DBusReader {
    pub(crate) const fn new(seq: u64) -> Self {
        Self {
            state: State::ReadHeader { bytes_read: 0 },
            seq,
        }
    }

    pub(crate) fn wants<'readbuf>(
        &self,
        buf: &'readbuf mut Vec<u8>,
    ) -> Option<DBusWants<'readbuf, 'static>> {
        match self.state {
            State::ReadHeader { bytes_read } => {
                buf.resize(HEADER_LEN, 0);
                let remainder = &mut buf[bytes_read..HEADER_LEN];
                Some(DBusWants::Read {
                    buf: remainder,
                    seq: self.seq,
                })
            }

            State::ReadBody {
                bytes_read,
                message_len,
            } => {
                buf.resize(message_len, 0);
                let remainder = &mut buf[bytes_read..message_len];
                Some(DBusWants::Read {
                    buf: remainder,
                    seq: self.seq,
                })
            }
            State::Dead => None,
        }
    }

    pub(crate) fn satisfy_read(
        &mut self,
        len: usize,
        buf: &[u8],
    ) -> Result<Option<usize>, DBusError> {
        match &mut self.state {
            State::ReadHeader { bytes_read } => {
                *bytes_read += len;
                self.seq += 1;

                if *bytes_read == HEADER_LEN {
                    let header = Header::from_bytes(buf)?;

                    let header_fields_len = (header.header_fields_len as usize).next_multiple_of(8);
                    let message_len = HEADER_LEN
                        .checked_add(header_fields_len)
                        .and_then(|len| len.checked_add(header.body_len as usize))
                        .ok_or(DBusError::MessageLengthOverflow)?;

                    self.state = State::ReadBody {
                        message_len,
                        bytes_read: HEADER_LEN,
                    };
                }

                Ok(None)
            }

            State::ReadBody {
                message_len,
                bytes_read,
            } => {
                *bytes_read += len;
                self.seq += 1;

                if *bytes_read == *message_len {
                    let len = *message_len;
                    self.state = State::ReadHeader { bytes_read: 0 };

                    Ok(Some(len))
                } else {
                    Ok(None)
                }
            }

            State::Dead => Ok(None),
        }
    }

    pub(crate) const fn stop(&mut self) {
        self.state = State::Dead;
    }
}
