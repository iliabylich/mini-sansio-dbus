use crate::{DBusConnectorWants, DBusError};

#[derive(Debug, Clone, Copy)]
enum State {
    WriteZero,
    WriteAuthExternal { bytes_written: usize },
    ReadData { bytes_read: usize },
    WriteData { bytes_written: usize },
    ReadGUID { bytes_read: usize },
    WriteBegin { bytes_written: usize },
}

const ZERO: &[u8] = b"\0";
const AUTH_EXTERNAL: &[u8] = b"AUTH EXTERNAL\r\n";
const DATA: &[u8] = b"DATA\r\n";
const BEGIN: &[u8] = b"BEGIN\r\n";
const GUID_LENGTH: usize = 37;

/// A state machine type to authenticate in `DBus`
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct DBusConnector {
    state: State,
    seq: u64,
}

impl Default for DBusConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl DBusConnector {
    /// Constructor
    pub const fn new() -> Self {
        Self {
            state: State::WriteZero,
            seq: 0,
        }
    }

    /// Returns what connector wants at the moment
    ///
    /// Returned value is either a read or a write operation that must be converted to a syscall of some sort,
    /// performed by the caller, and its result must be delivered back with either `satisfy_read` or `satisfy_write`
    ///
    /// # Errors
    ///
    /// Returns an error if given `readbuf` is too short
    pub fn wants<'r>(
        &self,
        buf: &'r mut [u8],
    ) -> Result<DBusConnectorWants<'r, 'static>, DBusError> {
        match self.state {
            State::WriteZero => Ok(DBusConnectorWants::Write {
                buf: ZERO,
                seq: self.seq,
            }),

            State::WriteAuthExternal { bytes_written } => {
                let remainder = AUTH_EXTERNAL
                    .get(bytes_written..)
                    .ok_or(DBusError::InternalError)?;
                Ok(DBusConnectorWants::Write {
                    buf: remainder,
                    seq: self.seq,
                })
            }

            State::ReadData { bytes_read } => {
                let buf = buf
                    .get_mut(bytes_read..DATA.len())
                    .ok_or(DBusError::ReadBufIsTooShort)?;
                Ok(DBusConnectorWants::Read { buf, seq: self.seq })
            }

            State::WriteData { bytes_written } => {
                let remainder = DATA.get(bytes_written..).ok_or(DBusError::InternalError)?;
                Ok(DBusConnectorWants::Write {
                    buf: remainder,
                    seq: self.seq,
                })
            }

            State::ReadGUID { bytes_read } => {
                let buf = buf
                    .get_mut(bytes_read..GUID_LENGTH)
                    .ok_or(DBusError::ReadBufIsTooShort)?;
                Ok(DBusConnectorWants::Read { buf, seq: self.seq })
            }

            State::WriteBegin { bytes_written } => {
                let remainder = BEGIN.get(bytes_written..).ok_or(DBusError::InternalError)?;
                Ok(DBusConnectorWants::Write {
                    buf: remainder,
                    seq: self.seq,
                })
            }
        }
    }

    /// Satisfies previously requested read operation
    ///
    /// # Errors
    ///
    /// Fails is operation is not the one that was last returned from `wants`
    pub fn satisfy_read(&mut self, len: usize, buf: &[u8]) -> Result<(), DBusError> {
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

    /// Satisfies previously requested write operation
    ///
    /// # Errors
    ///
    /// Fails is operation is not the one that was last returned from `wants`
    pub fn satisfy_write(&mut self, len: usize) -> Result<Option<u64>, DBusError> {
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
                    Ok(Some(self.seq))
                } else {
                    Ok(None)
                }
            }
            _ => Err(DBusError::InternalError),
        }
    }
}
