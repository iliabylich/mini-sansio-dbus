use crate::{DBusError, DBusWantsRead, types::Header};

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
}

impl DBusReader {
    pub(crate) const fn new(seq: u64) -> Self {
        Self {
            state: State::ReadHeader { bytes_read: 0 },
            seq,
        }
    }

    pub(crate) fn wants<'r>(&self, buf: &'r mut [u8]) -> Result<DBusWantsRead<'r>, DBusError> {
        match self.state {
            State::ReadHeader { bytes_read } => {
                let remainder = buf
                    .get_mut(bytes_read..HEADER_LEN)
                    .ok_or(DBusError::ReadBufIsTooShort)?;
                Ok(DBusWantsRead {
                    buf: remainder,
                    seq: self.seq,
                })
            }

            State::ReadBody {
                bytes_read,
                message_len,
            } => {
                let remainder = buf
                    .get_mut(bytes_read..message_len)
                    .ok_or(DBusError::ReadBufIsTooShort)?;
                Ok(DBusWantsRead {
                    buf: remainder,
                    seq: self.seq,
                })
            }
        }
    }

    pub(crate) fn satisfy_read(
        &mut self,
        len: usize,
        buf: &[u8],
    ) -> Result<Option<usize>, DBusError> {
        match &mut self.state {
            State::ReadHeader { bytes_read } => {
                *bytes_read = bytes_read
                    .checked_add(len)
                    .ok_or(DBusError::InternalError)?;
                self.seq = self.seq.checked_add(1).ok_or(DBusError::InternalError)?;

                if *bytes_read == HEADER_LEN {
                    let header = Header::from_bytes(buf)?;

                    let header_fields_len = (header.header_fields_len as usize)
                        .checked_next_multiple_of(8)
                        .ok_or(DBusError::MessageLengthOverflow)?;
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
                *bytes_read = bytes_read
                    .checked_add(len)
                    .ok_or(DBusError::InternalError)?;
                self.seq = self.seq.checked_add(1).ok_or(DBusError::InternalError)?;

                if *bytes_read == *message_len {
                    let len = *message_len;
                    self.state = State::ReadHeader { bytes_read: 0 };

                    Ok(Some(len))
                } else {
                    Ok(None)
                }
            }
        }
    }
}
