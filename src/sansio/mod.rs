use crate::{DBusError, DBusWants, IncomingMessage};
use connector::DBusConnector;
use reader::DBusReader;
use rustix::net::SocketAddrUnix;
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
    const fn new(addr: SocketAddrUnix) -> Self {
        Self {
            state: State::Connecting(DBusConnector::new(addr)),
        }
    }

    /// Constructs a dummy connection that doesn't "want" anything from you
    ///
    /// Can be used as a fallback if things go wrong.
    pub fn dummy() -> Self {
        Self {
            state: State::Connecting(DBusConnector::dummy()),
        }
    }

    /// Constructs a new session connection
    ///
    /// # Errors
    ///
    /// Fails if `$DBUS_SESSION_BUS_ADDRESS` env variable isn't set or contains NULL.
    pub fn new_session() -> Result<Self, DBusError> {
        let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")
            .map_err(|_| DBusError::NoSessionBusAddress)?;
        let (_, path) = address
            .split_once('=')
            .ok_or(DBusError::MalformedSessionBusAddress)?;

        let addr = SocketAddrUnix::new(path.as_bytes()).map_err(|_| DBusError::DBusPathWithNull)?;

        Ok(Self::new(addr))
    }

    /// Constructs a new system connection
    ///
    /// # Errors
    ///
    /// Fails if session UNIX address contains NULL.
    pub fn new_system() -> Result<Self, DBusError> {
        fn socket_path() -> String {
            std::env::var("DBUS_SYSTEM_BUS_ADDRESS")
                .ok()
                .and_then(|address| address.split_once('=').map(|(_, path)| path.to_string()))
                .unwrap_or_else(|| String::from("/var/run/dbus/system_bus_socket"))
        }

        let addr = SocketAddrUnix::new(socket_path().as_bytes())
            .map_err(|_| DBusError::DBusPathWithNull)?;

        Ok(Self::new(addr))
    }

    /// Returns what connection wants at the moment
    ///
    /// Returned value must be parsed and converted to a syscall of some sort
    pub fn wants<'readbuf, 'writebuf>(
        &mut self,
        queue: &'writebuf mut DBusQueue,
        readbuf: &'readbuf mut Vec<u8>,
    ) -> Option<DBusWants<'readbuf, 'writebuf>> {
        match &mut self.state {
            State::Connecting(connector) => connector.wants(readbuf),
            State::Ready { reader, writer } => match (reader.wants(readbuf), writer.wants(queue)) {
                (
                    Some(DBusWants::Read {
                        buf: readbuf,
                        seq: readseq,
                    }),
                    Some(DBusWants::Write {
                        buf: writebuf,
                        seq: writeseq,
                        ..
                    }),
                ) => Some(DBusWants::ReadWrite {
                    readbuf,
                    readseq,
                    writebuf,
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

    /// Notifies about completion of a `socket()` operation
    ///
    /// # Errors
    ///
    /// Fails is operation is not the one that was last returned from `wants`
    pub fn satisfy_socket(&mut self) -> Result<(), DBusError> {
        let State::Connecting(connector) = &mut self.state else {
            return Err(DBusError::InternalError);
        };

        connector.satisfy_socket()?;
        Ok(())
    }

    /// Notifies about completion of a `connect()` operation
    ///
    /// # Errors
    ///
    /// Fails is operation is not the one that was last returned from `wants`
    pub fn satisfy_connect(&mut self) -> Result<(), DBusError> {
        let State::Connecting(connector) = &mut self.state else {
            return Err(DBusError::InternalError);
        };

        connector.satisfy_connect()?;
        Ok(())
    }

    /// Notifies about completion of a `read()` operation
    ///
    /// # Errors
    ///
    /// Fails is operation is not the one that was last returned from `wants`
    pub fn satisfy_read<'readbuf>(
        &mut self,
        len: usize,
        readbuf: &'readbuf [u8],
    ) -> Result<Option<IncomingMessage<'readbuf>>, DBusError> {
        match &mut self.state {
            State::Connecting(connector) => {
                connector.satisfy_read(len, readbuf)?;
                Ok(None)
            }
            State::Ready { reader, .. } => {
                let Some(len) = reader.satisfy_read(len, readbuf)? else {
                    return Ok(None);
                };

                let buf = &readbuf[..len];

                let message = IncomingMessage::new(buf)?;
                Ok(Some(message))
            }
        }
    }

    /// Notifies about completion of a `write()` operation
    ///
    /// # Errors
    ///
    /// Fails is operation is not the one that was last returned from `wants`
    pub fn satisfy_write(&mut self, len: usize, queue: &mut DBusQueue) -> Result<(), DBusError> {
        match &mut self.state {
            State::Connecting(connector) => {
                if let Some(seq) = connector.satisfy_write(len)? {
                    self.state = State::Ready {
                        reader: DBusReader::new(seq),
                        writer: DBusWriter::new(seq),
                    };
                }
                Ok(())
            }
            State::Ready { writer, .. } => {
                writer.satisfy_write(len, queue)?;
                Ok(())
            }
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
