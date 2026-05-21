use crate::{DBusError, DBusWants, sansio::DBusQueue};

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

    pub(crate) fn wants<'writebuf>(
        &self,
        queue: &'writebuf DBusQueue,
    ) -> Option<DBusWants<'static, 'writebuf>> {
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

    pub(crate) fn satisfy_write(
        &mut self,
        len: usize,
        queue: &mut DBusQueue,
    ) -> Result<(), DBusError> {
        match &mut self.state {
            State::Writing { bytes_written } => {
                let buf = queue.front().ok_or(DBusError::InternalError)?;

                *bytes_written += len;
                self.seq += 1;

                if *bytes_written == buf.len() {
                    *bytes_written = 0;
                    queue.pop_front();
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
