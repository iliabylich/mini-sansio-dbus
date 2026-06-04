use crate::{DBusError, DBusWantsWrite, sansio::OutgoingQueue};

pub(crate) struct DBusWriter {
    bytes_written: usize,
    seq: u64,
}

impl DBusWriter {
    pub(crate) const fn new(seq: u64) -> Self {
        Self {
            bytes_written: 0,
            seq,
        }
    }

    pub(crate) fn wants<'w, Q>(&self, queue: &'w Q) -> Result<Option<DBusWantsWrite<'w>>, DBusError>
    where
        Q: OutgoingQueue,
    {
        let Some(buf) = queue.peek() else {
            return Ok(None);
        };
        let remainder = buf
            .get(self.bytes_written..)
            .ok_or(DBusError::InternalError)?;
        Ok(Some(DBusWantsWrite {
            buf: remainder,
            seq: self.seq,
        }))
    }

    pub(crate) fn satisfy_write<Q>(&mut self, len: usize, queue: &mut Q) -> Result<(), DBusError>
    where
        Q: OutgoingQueue,
    {
        let buf = queue.peek().ok_or(DBusError::InternalError)?;

        self.bytes_written = self
            .bytes_written
            .checked_add(len)
            .ok_or(DBusError::InternalError)?;
        self.seq = self.seq.checked_add(1).ok_or(DBusError::InternalError)?;

        if self.bytes_written == buf.len() {
            self.bytes_written = 0;
            queue.pop();
        }

        Ok(())
    }
}
