use crate::{DBusError, DBusSatisfy, DBusWants, sansio::DBusQueue};

pub(crate) struct DBusWriter {
    fd: i32,
    state: State,
    seq: u64,
}

#[derive(Debug, Clone, Copy)]
enum State {
    Writing { bytes_written: usize },
    Dead,
}

impl DBusWriter {
    pub(crate) const fn new(fd: i32, seq: u64) -> Self {
        Self {
            fd,
            state: State::Writing { bytes_written: 0 },
            seq,
        }
    }

    pub(crate) fn wants(&self, queue: &DBusQueue) -> Option<DBusWants> {
        match self.state {
            State::Writing { bytes_written } => {
                let buf = queue.front()?;
                let remainder = &buf[bytes_written..];
                Some(DBusWants::Write {
                    fd: self.fd,
                    buf: remainder.as_ptr(),
                    len: remainder.len(),
                    seq: self.seq,
                })
            }
            State::Dead => None,
        }
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: DBusSatisfy,
        res: i32,
        queue: &mut DBusQueue,
    ) -> Result<(), DBusError> {
        if satisfy != DBusSatisfy::Write {
            return Err(DBusError::InternalError(format!(
                "unexpected satisfy {satisfy:?} (expected Write)"
            )));
        }

        match &mut self.state {
            State::Writing { bytes_written } => {
                let buf = queue.front().ok_or_else(|| {
                    DBusError::InternalError(
                        "empty Queue, can't process Satisfy::Write".to_string(),
                    )
                })?;

                if res < 0 {
                    return Err(DBusError::WriteError(format!("Write failed: {res}")));
                }
                *bytes_written += res as usize;
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
