use crate::{DBusError, Satisfy, Wants};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

#[derive(Debug, Clone, Copy)]
enum State {
    ReadyTo(Action),
    WaitingFor(Action),
    Done,
    Dead,
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Socket,
    Connect,
    WriteZero,
    WriteAuthExternal,
    ReadData,
    WriteData,
    ReadGUID,
    WriteBegin,
}

pub(crate) struct DBusConnector {
    fd: i32,
    state: State,
    addr: sockaddr_un,
    buf: [u8; 100],
}

impl DBusConnector {
    pub(crate) const fn new(addr: sockaddr_un) -> Self {
        Self {
            fd: -1,
            state: State::ReadyTo(Action::Socket),
            addr,
            buf: [0; _],
        }
    }

    pub(crate) const fn dummy() -> Self {
        Self {
            fd: -1,
            state: State::Dead,
            addr: sockaddr_un {
                sun_family: 0,
                sun_path: [0; _],
            },
            buf: [0; _],
        }
    }

    pub(crate) const fn wants(&mut self) -> Option<Wants> {
        let State::ReadyTo(action) = self.state else {
            return None;
        };

        let wants = match action {
            Action::Socket => Wants::Socket {
                domain: AF_UNIX,
                r#type: SOCK_STREAM,
            },

            Action::Connect => Wants::Connect {
                fd: self.fd,
                addr: (&raw const self.addr).cast::<sockaddr>(),
                addrlen: size_of::<sockaddr_un>() as u32,
            },

            Action::WriteZero => {
                let buf = b"\0";
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }

            Action::WriteAuthExternal => {
                let buf = b"AUTH EXTERNAL\r\n";
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }

            Action::ReadData | Action::ReadGUID => Wants::Read {
                fd: self.fd,
                buf: self.buf.as_mut_ptr(),
                len: self.buf.len(),
            },

            Action::WriteData => {
                let buf = b"DATA\r\n";
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }

            Action::WriteBegin => {
                let buf = b"BEGIN\r\n";
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }
        };
        self.state = State::WaitingFor(action);
        Some(wants)
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<i32>, DBusError> {
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
            (Action::Socket, Satisfy::Socket) => {
                if res < 0 {
                    return Err(DBusError::ConnectError(format!("Socket failed: {res}")));
                }
                self.fd = res;
                self.state = State::ReadyTo(Action::Connect);
                Ok(None)
            }

            (Action::Connect, Satisfy::Connect) => {
                if res < 0 {
                    return Err(DBusError::ConnectError(format!("Connect failed: {res}")));
                }
                self.state = State::ReadyTo(Action::WriteZero);
                Ok(None)
            }

            (Action::WriteZero, Satisfy::Write) => {
                if res < 0 {
                    return Err(DBusError::ConnectError(format!("Write failed: {res}")));
                }
                let bytes_written = res as usize;
                if bytes_written != b"\0".len() {
                    return Err(DBusError::ConnectError(format!(
                        "Write failed, got {bytes_written} bytes written"
                    )));
                }
                self.state = State::ReadyTo(Action::WriteAuthExternal);
                Ok(None)
            }

            (Action::WriteAuthExternal, Satisfy::Write) => {
                if res < 0 {
                    return Err(DBusError::ConnectError(format!(
                        "WriteAuthExternal failed: {res}"
                    )));
                }
                let bytes_written = res as usize;
                if bytes_written != b"AUTH EXTERNAL\r\n".len() {
                    return Err(DBusError::ConnectError(format!(
                        "Write failed, got {bytes_written} bytes written"
                    )));
                }
                self.state = State::ReadyTo(Action::ReadData);
                Ok(None)
            }

            (Action::ReadData, Satisfy::Read) => {
                if res < 0 {
                    return Err(DBusError::ConnectError(format!("ReadData failed: {res}")));
                }
                let bytes_read = res as usize;
                if &self.buf[..bytes_read] != b"DATA\r\n" {
                    return Err(DBusError::ConnectError(format!(
                        "ReadData failed: expected to receive DATA, got {:?}",
                        &self.buf[..bytes_read]
                    )));
                }
                self.state = State::ReadyTo(Action::WriteData);
                Ok(None)
            }

            (Action::WriteData, Satisfy::Write) => {
                if res < 0 {
                    return Err(DBusError::ConnectError(format!("WriteData failed: {res}")));
                }
                let bytes_written = res as usize;
                if bytes_written != b"DATA\r\n".len() {
                    return Err(DBusError::ConnectError(format!(
                        "Write failed, got {bytes_written} bytes written"
                    )));
                }
                self.state = State::ReadyTo(Action::ReadGUID);
                Ok(None)
            }

            (Action::ReadGUID, Satisfy::Read) => {
                if res <= 0 {
                    return Err(DBusError::ConnectError(format!("ReadGUID failed: {res}")));
                }
                self.state = State::ReadyTo(Action::WriteBegin);
                Ok(None)
            }

            (Action::WriteBegin, Satisfy::Write) => {
                if res < 0 {
                    return Err(DBusError::ConnectError(format!("WriteBegin failed: {res}")));
                }
                let bytes_written = res as usize;
                if bytes_written != b"BEGIN\r\n".len() {
                    return Err(DBusError::ConnectError(format!(
                        "Write failed, got {bytes_written} bytes written"
                    )));
                }
                self.state = State::Done;
                Ok(Some(self.fd))
            }

            (state, satisfy) => Err(DBusError::InternalError(format!(
                "malformed state: {state:?} vs {satisfy:?}"
            ))),
        }
    }

    pub(crate) const fn stop(&mut self) {
        self.state = State::Dead;
    }
}
