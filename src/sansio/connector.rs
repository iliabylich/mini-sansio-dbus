use crate::{DBusError, DBusSatisfy, DBusWants};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

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
    fd: i32,
    state: State,
    addr: sockaddr_un,
    buf: [u8; 100],
    seq: u64,
}

impl DBusConnector {
    pub(crate) const fn new(addr: sockaddr_un) -> Self {
        Self {
            fd: -1,
            state: State::Socket,
            addr,
            buf: [0; _],
            seq: 0,
        }
    }

    pub(crate) const fn dummy() -> Self {
        Self {
            fd: -1,
            state: State::Stopped,
            addr: sockaddr_un {
                sun_family: 0,
                sun_path: [0; _],
            },
            buf: [0; _],
            seq: 0,
        }
    }

    pub(crate) fn wants(&mut self) -> Option<DBusWants> {
        match self.state {
            State::Socket => Some(DBusWants::Socket {
                domain: AF_UNIX,
                r#type: SOCK_STREAM,
                seq: self.seq,
            }),

            State::Connect => Some(DBusWants::Connect {
                fd: self.fd,
                addr: (&raw const self.addr).cast::<sockaddr>(),
                addrlen: size_of::<sockaddr_un>() as u32,
                seq: self.seq,
            }),

            State::WriteZero => Some(DBusWants::Write {
                fd: self.fd,
                buf: ZERO.as_ptr(),
                len: ZERO.len(),
                seq: self.seq,
            }),

            State::WriteAuthExternal { bytes_written } => {
                let remainder = &AUTH_EXTERNAL[bytes_written..];
                Some(DBusWants::Write {
                    fd: self.fd,
                    buf: remainder.as_ptr(),
                    len: remainder.len(),
                    seq: self.seq,
                })
            }

            State::ReadData { bytes_read } => {
                let remainder = &mut self.buf[bytes_read..DATA.len()];
                Some(DBusWants::Read {
                    fd: self.fd,
                    buf: remainder.as_mut_ptr(),
                    len: remainder.len(),
                    seq: self.seq,
                })
            }

            State::WriteData { bytes_written } => {
                let remainder = &DATA[bytes_written..];
                Some(DBusWants::Write {
                    fd: self.fd,
                    buf: remainder.as_ptr(),
                    len: remainder.len(),
                    seq: self.seq,
                })
            }

            State::ReadGUID { bytes_read } => {
                let remainder = &mut self.buf[bytes_read..GUID_LENGTH];
                Some(DBusWants::Read {
                    fd: self.fd,
                    buf: remainder.as_mut_ptr(),
                    len: remainder.len(),
                    seq: self.seq,
                })
            }

            State::WriteBegin { bytes_written } => {
                let remainder = &BEGIN[bytes_written..];
                Some(DBusWants::Write {
                    fd: self.fd,
                    buf: remainder.as_ptr(),
                    len: remainder.len(),
                    seq: self.seq,
                })
            }

            State::Stopped => None,
        }
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: DBusSatisfy,
        res: i32,
    ) -> Result<Option<(i32, u64)>, DBusError> {
        match (&mut self.state, satisfy) {
            (State::Socket, DBusSatisfy::Socket) => {
                if res < 0 {
                    return Err(DBusError::ConnectError(format!("Socket failed: {res}")));
                }
                self.fd = res;
                self.state = State::Connect;
                self.seq += 1;
                Ok(None)
            }

            (State::Connect, DBusSatisfy::Connect) => {
                if res < 0 {
                    return Err(DBusError::ConnectError(format!("Connect failed: {res}")));
                }
                self.state = State::WriteZero;
                self.seq += 1;
                Ok(None)
            }

            (State::WriteZero, DBusSatisfy::Write) => {
                if res < 0 {
                    return Err(DBusError::ConnectError(format!("Write failed: {res}")));
                }
                if res == 1 {
                    self.state = State::WriteAuthExternal { bytes_written: 0 };
                    self.seq += 1;
                }
                Ok(None)
            }

            (State::WriteAuthExternal { bytes_written }, DBusSatisfy::Write) => {
                if res < 0 {
                    return Err(DBusError::ConnectError(format!(
                        "WriteAuthExternal failed: {res}"
                    )));
                }
                *bytes_written += res as usize;
                self.seq += 1;

                let remainder = &AUTH_EXTERNAL[*bytes_written..];
                if remainder.is_empty() {
                    self.state = State::ReadData { bytes_read: 0 };
                }
                Ok(None)
            }

            (State::ReadData { bytes_read }, DBusSatisfy::Read) => {
                if res < 0 {
                    return Err(DBusError::ConnectError(format!("ReadData failed: {res}")));
                }
                *bytes_read += res as usize;
                self.seq += 1;

                let remainder = &self.buf[*bytes_read..DATA.len()];
                if remainder.is_empty() {
                    self.state = State::WriteData { bytes_written: 0 };
                }
                Ok(None)
            }

            (State::WriteData { bytes_written }, DBusSatisfy::Write) => {
                if res < 0 {
                    return Err(DBusError::ConnectError(format!("WriteData failed: {res}")));
                }
                *bytes_written += res as usize;
                self.seq += 1;

                let remainder = &DATA[*bytes_written..];
                if remainder.is_empty() {
                    self.state = State::ReadGUID { bytes_read: 0 };
                }
                Ok(None)
            }

            (State::ReadGUID { bytes_read }, DBusSatisfy::Read) => {
                if res < 0 {
                    return Err(DBusError::ConnectError(format!("ReadGUID failed: {res}")));
                }
                *bytes_read += res as usize;
                self.seq += 1;

                if *bytes_read == GUID_LENGTH {
                    self.state = State::WriteBegin { bytes_written: 0 };
                }
                Ok(None)
            }

            (State::WriteBegin { bytes_written }, DBusSatisfy::Write) => {
                if res < 0 {
                    return Err(DBusError::ConnectError(format!("WriteBegin failed: {res}")));
                }
                *bytes_written += res as usize;
                self.seq += 1;

                let remainder = &BEGIN[*bytes_written..];
                if remainder.is_empty() {
                    self.state = State::Stopped;
                    Ok(Some((self.fd, self.seq)))
                } else {
                    Ok(None)
                }
            }

            (state, satisfy) => Err(DBusError::InternalError(format!(
                "malformed state: {state:?} vs {satisfy:?}"
            ))),
        }
    }

    pub(crate) const fn stop(&mut self) {
        self.state = State::Stopped;
    }
}
