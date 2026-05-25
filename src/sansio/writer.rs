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

    pub(crate) fn wants<'w, 'q, Q>(&self, queue: &'w Q) -> Option<DBusWants<'static, 'w>>
    where
        Q: OutgoingQueue<'q>,
    {
        match self.state {
            State::Writing { bytes_written } => {
                let buf = queue.peek()?;
                let remainder = buf.get(bytes_written..)?;
                Some(DBusWants::Write {
                    buf: remainder,
                    seq: self.seq,
                })
            }
            State::Dead => None,
        }
    }

    pub(crate) fn satisfy_write<'q, Q>(
        &mut self,
        len: usize,
        queue: &mut Q,
    ) -> Result<(), DBusError>
    where
        Q: OutgoingQueue<'q>,
    {
        match &mut self.state {
            State::Writing { bytes_written } => {
                let buf = queue.peek().ok_or(DBusError::InternalError)?;

                *bytes_written = bytes_written
                    .checked_add(len)
                    .ok_or(DBusError::InternalError)?;
                self.seq = self.seq.checked_add(1).ok_or(DBusError::InternalError)?;

                if *bytes_written == buf.len() {
                    *bytes_written = 0;
                    queue.pop();
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
