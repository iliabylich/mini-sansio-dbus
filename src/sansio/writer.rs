use crate::{DBusError, DBusWants, sansio::OutgoingQueue};

pub(crate) struct DBusWriter {
    state: State,
    seq: u64,
}

#[derive(Debug, Clone, Copy)]
enum State {
    Writing { bytes_written: usize },
    Dead,
}

impl DBusWriter {
    pub(crate) const fn new(seq: u64) -> Self {
        Self {
            state: State::Writing { bytes_written: 0 },
            seq,
        }
    }

    pub(crate) fn wants<'writebuf, Q>(
        &self,
        queue: &'writebuf Q,
    ) -> Option<DBusWants<'static, 'writebuf>>
    where
        Q: OutgoingQueue,
    {
        match self.state {
            State::Writing { bytes_written } => {
                let buf = queue.front()?;
                let remainder = &buf[bytes_written..];
                Some(DBusWants::Write {
                    buf: remainder,
                    seq: self.seq,
                })
            }
            State::Dead => None,
        }
    }

    pub(crate) fn satisfy_write<Q>(&mut self, len: usize, queue: &mut Q) -> Result<(), DBusError>
    where
        Q: OutgoingQueue,
    {
        match &mut self.state {
            State::Writing { bytes_written } => {
                let buf = queue.front().ok_or(DBusError::InternalError)?;

                *bytes_written += len;
                self.seq += 1;

                if *bytes_written == buf.len() {
                    *bytes_written = 0;
                    let _ = queue.pop_front();
                }
            }
            State::Dead => {}
        }

        Ok(())
    }

    pub(crate) const fn stop(&mut self) {
        self.state = State::Dead;
    }
}
