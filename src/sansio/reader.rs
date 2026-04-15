use crate::{DBusError, Satisfy, Wants, types::Header};

const HEADER_LEN: usize = core::mem::size_of::<Header>();

pub(crate) struct DBusReader {
    fd: i32,
    bytes_read: usize,
    message_len: usize,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    ReadyTo(Action),
    WaitingFor(Action),
    Dead,
}

#[derive(Debug, Clone, Copy)]
enum Action {
    ReadHeader,
    ReadBody,
}

impl DBusReader {
    pub(crate) fn new(fd: i32) -> Self {
        Self {
            fd,
            bytes_read: 0,
            message_len: 0,
            state: State::ReadyTo(Action::ReadHeader),
        }
    }

    pub(crate) fn wants(&mut self, buf: &mut Vec<u8>) -> Option<Wants> {
        let State::ReadyTo(action) = self.state else {
            return None;
        };

        let wants = match action {
            Action::ReadHeader => {
                buf.resize(HEADER_LEN, 0);
                Wants::Read {
                    fd: self.fd,
                    buf: buf.as_mut_ptr(),
                    len: HEADER_LEN,
                }
            }

            Action::ReadBody => {
                buf.resize(self.message_len, 0);
                let buf = &mut buf[self.bytes_read..self.message_len];
                Wants::Read {
                    fd: self.fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                }
            }
        };
        self.state = State::WaitingFor(action);
        Some(wants)
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
        buf: &[u8],
    ) -> Result<Option<usize>, DBusError> {
        let action = match self.state {
            State::WaitingFor(action) => action,
            State::Dead => return Ok(None),
            state => {
                return Err(DBusError::InternalError(format!(
                    "malformed state: {state:?} vs {satisfy:?}"
                )));
            }
        };

        match (action, satisfy) {
            (Action::ReadHeader, Satisfy::Read) => {
                if res == 0 {
                    return Ok(None);
                }
                if res <= 0 {
                    return Err(DBusError::ReadError(format!("ReadHeader failed: {res}")));
                }
                let bytes_read = res as usize;
                if bytes_read != HEADER_LEN {
                    return Err(DBusError::ReadError(format!(
                        "ReadHeader: got {bytes_read} bytes instead of {HEADER_LEN}"
                    )));
                }
                self.bytes_read += bytes_read;

                let header = Header::from_bytes(buf)?;

                let header_fields_len = (header.header_fields_len as usize).next_multiple_of(8);
                let message_len = HEADER_LEN
                    .checked_add(header_fields_len)
                    .and_then(|len| len.checked_add(header.body_len as usize))
                    .ok_or(DBusError::MessageLengthOverflow)?;

                self.message_len = message_len;
                self.state = State::ReadyTo(Action::ReadBody);

                Ok(None)
            }

            (Action::ReadBody, Satisfy::Read) => {
                if res <= 0 {
                    return Err(DBusError::ReadError(format!("ReadBody failed: {res}")));
                }
                let bytes_read = res as usize;
                self.bytes_read += bytes_read;

                if self.bytes_read == self.message_len {
                    let message_len = self.message_len;

                    self.bytes_read = 0;
                    self.message_len = 0;
                    self.state = State::ReadyTo(Action::ReadHeader);

                    return Ok(Some(message_len));
                } else {
                    self.state = State::ReadyTo(Action::ReadBody);
                }

                Ok(None)
            }

            (_, _) => Err(DBusError::InternalError(format!(
                "malformed state: {action:?} vs {satisfy:?}"
            ))),
        }
    }

    pub(crate) fn stop(&mut self) {
        self.state = State::Dead;
    }
}
