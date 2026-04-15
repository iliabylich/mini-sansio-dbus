use crate::{DBusError, IncomingMessage, Satisfy, Wants};
use connector::DBusConnector;
use libc::{AF_UNIX, sockaddr_un};
use reader::DBusReader;
use writer::DBusWriter;

pub use queue::DBusQueue;

mod connector;
mod queue;
mod reader;
mod writer;

pub struct DBusConnection {
    state: State,
}

enum State {
    Connecting(DBusConnector),
    Ready {
        reader: DBusReader,
        writer: DBusWriter,
    },
}

impl DBusConnection {
    fn new(addr: sockaddr_un) -> Self {
        Self {
            state: State::Connecting(DBusConnector::new(addr)),
        }
    }

    pub fn dummy() -> Self {
        Self {
            state: State::Connecting(DBusConnector::dummy()),
        }
    }

    pub fn new_session() -> Result<Self, DBusError> {
        let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")
            .map_err(|_| DBusError::NoSessionBusAddress)?;
        let (_, path) = address
            .split_once("=")
            .ok_or(DBusError::MalformedSessionBusAddress)?;

        let addr = new_unix_socket(path.as_bytes());

        Ok(Self::new(addr))
    }

    pub fn new_system() -> Self {
        fn socket_path() -> String {
            std::env::var("DBUS_SYSTEM_BUS_ADDRESS")
                .ok()
                .and_then(|address| address.split_once("=").map(|(_, path)| path.to_string()))
                .unwrap_or_else(|| String::from("/var/run/dbus/system_bus_socket"))
        }

        let addr = new_unix_socket(socket_path().as_bytes());

        Self::new(addr)
    }

    pub fn wants(&mut self, queue: &mut DBusQueue, readbuf: &mut Vec<u8>) -> Option<Wants> {
        match &mut self.state {
            State::Connecting(connector) => connector.wants(),
            State::Ready { reader, writer } => match (reader.wants(readbuf), writer.wants(queue)) {
                (
                    Some(Wants::Read {
                        fd,
                        buf: readbuf,
                        len: readlen,
                    }),
                    Some(Wants::Write {
                        buf: writebuf,
                        len: writelen,
                        ..
                    }),
                ) => Some(Wants::ReadWrite {
                    fd,
                    readbuf,
                    readlen,
                    writebuf,
                    writelen,
                }),

                (read, None) => read,
                (None, write) => write,
                other => {
                    unreachable!("bug: DBus reader/writer never want {other:?}")
                }
            },
        }
    }

    pub fn satisfy<'a>(
        &mut self,
        satisfy: Satisfy,
        res: i32,
        readbuf: &'a [u8],
        queue: &mut DBusQueue,
    ) -> Result<Option<IncomingMessage<'a>>, DBusError> {
        match &mut self.state {
            State::Connecting(connector) => {
                let Some(fd) = connector.satisfy(satisfy, res)? else {
                    return Ok(None);
                };

                self.state = State::Ready {
                    reader: DBusReader::new(fd),
                    writer: DBusWriter::new(fd, queue),
                };
                Ok(None)
            }

            State::Ready { reader, writer } => match satisfy {
                Satisfy::Read => {
                    let Some(len) = reader.satisfy(satisfy, res, readbuf)? else {
                        return Ok(None);
                    };
                    let buf = &readbuf[..len];

                    let message = IncomingMessage::new(buf)?;
                    Ok(Some(message))
                }

                Satisfy::Write => {
                    writer.satisfy(satisfy, res, queue)?;
                    Ok(None)
                }

                _ => Err(DBusError::InternalError(format!(
                    "DBus in r/w mode received unexpected satisfy: {satisfy:?}"
                ))),
            },
        }
    }

    pub fn stop(&mut self) {
        match &mut self.state {
            State::Connecting(connector) => connector.stop(),
            State::Ready { reader, writer } => {
                reader.stop();
                writer.stop();
            }
        }
    }
}

fn new_unix_socket(path: &[u8]) -> sockaddr_un {
    sockaddr_un {
        sun_family: AF_UNIX as u16,
        sun_path: {
            let mut out = [0; 108];
            for (idx, byte) in path.iter().enumerate() {
                out[idx] = *byte as i8;
            }
            out
        },
    }
}
