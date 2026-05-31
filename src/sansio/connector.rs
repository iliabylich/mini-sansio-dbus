use crate::{DBusError, DBusWants};
use rustix::net::{AddressFamily, SocketAddrUnix, SocketType};

#[derive(Debug, Clone, Copy)]
enum State {
    Socket,
    Connect,
    WriteZero,
    WriteAuthExternal { bytes_written: usize },
    ReadData { bytes_read: usize },
    WriteData { bytes_written: usize },
    ReadGUID { bytes_read: usize },
    WriteBegin { bytes_written: usize },

    Stopped,
}

const ZERO: &[u8] = b"\0";
const AUTH_EXTERNAL: &[u8] = b"AUTH EXTERNAL\r\n";
const DATA: &[u8] = b"DATA\r\n";
const BEGIN: &[u8] = b"BEGIN\r\n";
const GUID_LENGTH: usize = 37;

pub(crate) struct DBusConnector {
    state: State,
    addr: SocketAddrUnix,
    seq: u64,
}

impl DBusConnector {
    pub(crate) const fn new(addr: SocketAddrUnix) -> Self {
        Self {
            state: State::Socket,
            addr,
            seq: 0,
        }
    }

    pub(crate) fn dummy() -> Self {
        Self {
            state: State::Stopped,
            addr: SocketAddrUnix::new_unnamed(),
            seq: 0,
        }
    }

    pub(crate) fn wants<'r>(
        &self,
        buf: &'r mut [u8],
    ) -> Result<Option<DBusWants<'r, 'static>>, DBusError> {
        match self.state {
            State::Socket => Ok(Some(DBusWants::Socket {
                domain: AddressFamily::UNIX,
                r#type: SocketType::STREAM,
                seq: self.seq,
            })),

            State::Connect => Ok(Some(DBusWants::Connect {
                addr: self.addr.clone(),
                seq: self.seq,
            })),

            State::WriteZero => Ok(Some(DBusWants::Write {
                buf: ZERO,
                seq: self.seq,
            })),

            State::WriteAuthExternal { bytes_written } => {
                let remainder = AUTH_EXTERNAL
                    .get(bytes_written..)
                    .ok_or(DBusError::InternalError)?;
                Ok(Some(DBusWants::Write {
                    buf: remainder,
                    seq: self.seq,
                }))
            }

            State::ReadData { bytes_read } => {
                let buf = buf
                    .get_mut(bytes_read..DATA.len())
                    .ok_or(DBusError::ReadBufIsTooShort)?;
                Ok(Some(DBusWants::Read { buf, seq: self.seq }))
            }

            State::WriteData { bytes_written } => {
                let remainder = DATA.get(bytes_written..).ok_or(DBusError::InternalError)?;
                Ok(Some(DBusWants::Write {
                    buf: remainder,
                    seq: self.seq,
                }))
            }

            State::ReadGUID { bytes_read } => {
                let buf = buf
                    .get_mut(bytes_read..GUID_LENGTH)
                    .ok_or(DBusError::ReadBufIsTooShort)?;
                Ok(Some(DBusWants::Read { buf, seq: self.seq }))
            }

            State::WriteBegin { bytes_written } => {
                let remainder = BEGIN.get(bytes_written..).ok_or(DBusError::InternalError)?;
                Ok(Some(DBusWants::Write {
                    buf: remainder,
                    seq: self.seq,
                }))
            }

            State::Stopped => Ok(None),
        }
    }

    pub(crate) fn satisfy_socket(&mut self) -> Result<(), DBusError> {
        if !matches!(self.state, State::Socket) {
            return Err(DBusError::InternalError);
        }

        self.state = State::Connect;
        self.advance_seq()?;
        Ok(())
    }

    pub(crate) fn satisfy_connect(&mut self) -> Result<(), DBusError> {
        if !matches!(self.state, State::Connect) {
            return Err(DBusError::InternalError);
        }

        self.state = State::WriteZero;
        self.advance_seq()?;
        Ok(())
    }

    pub(crate) fn satisfy_read(&mut self, len: usize, buf: &[u8]) -> Result<(), DBusError> {
        match &mut self.state {
            State::ReadData { bytes_read } => {
                *bytes_read = bytes_read
                    .checked_add(len)
                    .ok_or(DBusError::InternalError)?;
                self.seq = self.seq.checked_add(1).ok_or(DBusError::InternalError)?;

                let remainder = buf
                    .get(*bytes_read..DATA.len())
                    .ok_or(DBusError::InternalError)?;
                if remainder.is_empty() {
                    self.state = State::WriteData { bytes_written: 0 };
                }
                Ok(())
            }
            State::ReadGUID { bytes_read } => {
                *bytes_read = bytes_read
                    .checked_add(len)
                    .ok_or(DBusError::InternalError)?;
                self.seq = self.seq.checked_add(1).ok_or(DBusError::InternalError)?;

                if *bytes_read == GUID_LENGTH {
                    self.state = State::WriteBegin { bytes_written: 0 };
                }
                Ok(())
            }
            _ => Err(DBusError::InternalError),
        }
    }

    pub(crate) fn satisfy_write(&mut self, len: usize) -> Result<Option<u64>, DBusError> {
        match &mut self.state {
            State::WriteZero => {
                if len == 1 {
                    self.state = State::WriteAuthExternal { bytes_written: 0 };
                    self.seq = self.seq.checked_add(1).ok_or(DBusError::InternalError)?;
                }
                Ok(None)
            }

            State::WriteAuthExternal { bytes_written } => {
                *bytes_written = bytes_written
                    .checked_add(len)
                    .ok_or(DBusError::InternalError)?;
                self.seq = self.seq.checked_add(1).ok_or(DBusError::InternalError)?;

                let remainder = AUTH_EXTERNAL
                    .get(*bytes_written..)
                    .ok_or(DBusError::InternalError)?;
                if remainder.is_empty() {
                    self.state = State::ReadData { bytes_read: 0 };
                }
                Ok(None)
            }
            State::WriteData { bytes_written } => {
                *bytes_written = bytes_written
                    .checked_add(len)
                    .ok_or(DBusError::InternalError)?;
                self.seq = self.seq.checked_add(1).ok_or(DBusError::InternalError)?;

                let remainder = DATA.get(*bytes_written..).ok_or(DBusError::InternalError)?;
                if remainder.is_empty() {
                    self.state = State::ReadGUID { bytes_read: 0 };
                }
                Ok(None)
            }
            State::WriteBegin { bytes_written } => {
                *bytes_written = bytes_written
                    .checked_add(len)
                    .ok_or(DBusError::InternalError)?;
                self.seq = self.seq.checked_add(1).ok_or(DBusError::InternalError)?;

                let remainder = BEGIN
                    .get(*bytes_written..)
                    .ok_or(DBusError::InternalError)?;
                if remainder.is_empty() {
                    self.state = State::Stopped;
                    Ok(Some(self.seq))
                } else {
                    Ok(None)
                }
            }
            _ => Err(DBusError::InternalError),
        }
    }

    pub(crate) const fn stop(&mut self) {
        self.state = State::Stopped;
    }

    fn advance_seq(&mut self) -> Result<(), DBusError> {
        self.seq = self.seq.checked_add(1).ok_or(DBusError::InternalError)?;
        Ok(())
    }
}
