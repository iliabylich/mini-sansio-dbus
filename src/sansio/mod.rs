use crate::{DBusError, DBusWants, IncomingMessage};
use connector::DBusConnector;
use reader::DBusReader;
use rustix::net::SocketAddrUnix;
use writer::DBusWriter;

pub use queue::{DBusSerial, OutgoingQueue};

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
    /// Takes `address` which is usually `$DBUS_SESSION_BUS_ADDRESS`.
    ///
    /// # Errors
    ///
    /// Fails if given address contains NULL.
    pub fn new_session(address: &str) -> Result<Self, DBusError> {
        let addr = SocketAddrUnix::new(address).map_err(|_| DBusError::DBusPathWithNull)?;
        Ok(Self::new(addr))
    }

    /// Constructs a new system connection
    ///
    /// Takes `address` which is usually either `$DBUS_SYSTEM_BUS_ADDRESS` or `/var/run/dbus/system_bus_socket`.
    ///
    /// # Errors
    ///
    /// Fails if given address contains NULL.
    pub fn new_system(address: &str) -> Result<Self, DBusError> {
        let addr = SocketAddrUnix::new(address).map_err(|_| DBusError::DBusPathWithNull)?;
        Ok(Self::new(addr))
    }

    /// Returns what connection wants at the moment
    ///
    /// Returned value must be parsed and converted to a syscall of some sort
    ///
    /// # Errors
    ///
    /// Returns an error if given `readbuf` is too short
    pub fn wants<'r, 'w, 'q, Q>(
        &mut self,
        queue: &'w Q,
        readbuf: &'r mut [u8],
    ) -> Result<Option<DBusWants<'r, 'w>>, DBusError>
    where
        Q: OutgoingQueue<'q>,
    {
        match &mut self.state {
            State::Connecting(connector) => connector.wants(readbuf),
            State::Ready { reader, writer } => {
                match (reader.wants(readbuf)?, writer.wants(queue)?) {
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
                    ) => Ok(Some(DBusWants::ReadWrite {
                        readbuf,
                        readseq,
                        writebuf,
                        writeseq,
                    })),

                    (read, None) => Ok(read),
                    (None, write) => Ok(write),
                    other => {
                        unreachable!("bug: DBus reader/writer never want {other:?}")
                    }
                }
            }
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
    pub fn satisfy_read<'r>(
        &mut self,
        len: usize,
        readbuf: &'r [u8],
    ) -> Result<Option<IncomingMessage<'r>>, DBusError> {
        match &mut self.state {
            State::Connecting(connector) => {
                connector.satisfy_read(len, readbuf)?;
                Ok(None)
            }
            State::Ready { reader, .. } => {
                let Some(len) = reader.satisfy_read(len, readbuf)? else {
                    return Ok(None);
                };

                let buf = readbuf.get(..len).ok_or(DBusError::InternalError)?;

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
    pub fn satisfy_write<'q, Q>(&mut self, len: usize, queue: &mut Q) -> Result<(), DBusError>
    where
        Q: OutgoingQueue<'q>,
    {
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
