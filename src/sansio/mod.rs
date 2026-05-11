use crate::{DBusError, DBusSatisfy, DBusWants, IncomingMessage};
use connector::DBusConnector;
use libc::{AF_UNIX, sockaddr_un};
use reader::DBusReader;
use writer::DBusWriter;

pub use queue::DBusQueue;

mod connector;
mod queue;
mod reader;
mod writer;

/// A `DBus` connection, the main type
#[must_use]
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
    const fn new(addr: sockaddr_un) -> Self {
        Self {
            state: State::Connecting(DBusConnector::new(addr)),
        }
    }

    /// Constructs a dummy connection that doesn't "want" anything from you
    ///
    /// Can be used as a fallback if things go wrong.
    pub const fn dummy() -> Self {
        Self {
            state: State::Connecting(DBusConnector::dummy()),
        }
    }

    /// Constructs a new session connection
    ///
    /// # Errors
    ///
    /// Fails if `$DBUS_SESSION_BUS_ADDRESS` env variable isn't set
    pub fn new_session() -> Result<Self, DBusError> {
        let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")
            .map_err(|_| DBusError::NoSessionBusAddress)?;
        let (_, path) = address
            .split_once('=')
            .ok_or(DBusError::MalformedSessionBusAddress)?;

        let addr = new_unix_socket(path.as_bytes());

        Ok(Self::new(addr))
    }

    /// Constructs a new system connection
    pub fn new_system() -> Self {
        fn socket_path() -> String {
            std::env::var("DBUS_SYSTEM_BUS_ADDRESS")
                .ok()
                .and_then(|address| address.split_once('=').map(|(_, path)| path.to_string()))
                .unwrap_or_else(|| String::from("/var/run/dbus/system_bus_socket"))
        }

        let addr = new_unix_socket(socket_path().as_bytes());

        Self::new(addr)
    }

    /// Returns what connection wants at the moment
    ///
    /// Returned value must be parsed and converted to a syscall of some sort
    pub fn wants(&mut self, queue: &mut DBusQueue, readbuf: &mut Vec<u8>) -> Option<DBusWants> {
        match &mut self.state {
            State::Connecting(connector) => connector.wants(),
            State::Ready { reader, writer } => match (reader.wants(readbuf), writer.wants(queue)) {
                (
                    Some(DBusWants::Read {
                        fd,
                        buf: readbuf,
                        len: readlen,
                        seq: readseq,
                    }),
                    Some(DBusWants::Write {
                        buf: writebuf,
                        len: writelen,
                        seq: writeseq,
                        ..
                    }),
                ) => Some(DBusWants::ReadWrite {
                    fd,
                    readbuf,
                    readlen,
                    readseq,
                    writebuf,
                    writelen,
                    writeseq,
                }),

                (read, None) => read,
                (None, write) => write,
                other => {
                    unreachable!("bug: DBus reader/writer never want {other:?}")
                }
            },
        }
    }

    /// Notifies about completion of a previously requested operation
    ///
    /// # Errors
    ///
    /// Fails is operation is not the one that was last returned from `wants`
    pub fn satisfy<'a>(
        &mut self,
        satisfy: DBusSatisfy,
        res: i32,
        readbuf: &'a [u8],
        queue: &mut DBusQueue,
    ) -> Result<Option<IncomingMessage<'a>>, DBusError> {
        match &mut self.state {
            State::Connecting(connector) => {
                let Some((fd, seq)) = connector.satisfy(satisfy, res)? else {
                    return Ok(None);
                };

                self.state = State::Ready {
                    reader: DBusReader::new(fd, seq),
                    writer: DBusWriter::new(fd, seq),
                };
                Ok(None)
            }

            State::Ready { reader, writer } => match satisfy {
                DBusSatisfy::Read => {
                    let Some(len) = reader.satisfy(satisfy, res, readbuf)? else {
                        return Ok(None);
                    };
                    let buf = &readbuf[..len];

                    let message = IncomingMessage::new(buf)?;
                    Ok(Some(message))
                }

                DBusSatisfy::Write => {
                    writer.satisfy(satisfy, res, queue)?;
                    Ok(None)
                }

                _ => Err(DBusError::InternalError(format!(
                    "DBus in r/w mode received unexpected satisfy: {satisfy:?}"
                ))),
            },
        }
    }

    /// Stops connection
    pub const fn stop(&mut self) {
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
