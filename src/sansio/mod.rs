use crate::{DBusError, DBusWantsRead, DBusWantsWrite, IncomingMessage};
use reader::DBusReader;
use writer::DBusWriter;

pub use connector::DBusConnector;
pub use queue::{DBusSerial, OutgoingQueue};

mod connector;
mod queue;
mod reader;
mod writer;

/// A `DBus` connection, the main type
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct DBusConnection {
    reader: DBusReader,
    writer: DBusWriter,
}

impl DBusConnection {
    /// Constructor
    pub const fn new(seq: u64) -> Self {
        Self {
            reader: DBusReader::new(seq),
            writer: DBusWriter::new(seq),
        }
    }

    /// Returns what connection wants at the moment
    ///
    /// Returned values must be parsed and converted to syscalls of some sort
    ///
    /// # Errors
    ///
    /// Returns an error if given `readbuf` is too short
    pub fn wants<'r, 'w, Q>(
        &self,
        queue: &'w Q,
        readbuf: &'r mut [u8],
    ) -> Result<(DBusWantsRead<'r>, Option<DBusWantsWrite<'w>>), DBusError>
    where
        Q: OutgoingQueue,
    {
        let read = self.reader.wants(readbuf)?;
        let write = self.writer.wants(queue)?;

        Ok((read, write))
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
        let Some(len) = self.reader.satisfy_read(len, readbuf)? else {
            return Ok(None);
        };

        let buf = readbuf.get(..len).ok_or(DBusError::InternalError)?;

        let message = IncomingMessage::new(buf)?;
        Ok(Some(message))
    }

    /// Notifies about completion of a `write()` operation
    ///
    /// # Errors
    ///
    /// Fails is operation is not the one that was last returned from `wants`
    pub fn satisfy_write<Q>(&mut self, len: usize, queue: &mut Q) -> Result<(), DBusError>
    where
        Q: OutgoingQueue,
    {
        self.writer.satisfy_write(len, queue)?;
        Ok(())
    }
}
