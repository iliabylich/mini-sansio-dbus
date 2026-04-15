use crate::{DBusError, Satisfy, Wants, sansio::DBusQueue};

pub(crate) struct DBusWriter {
    fd: i32,
    current: Option<Vec<u8>>,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    ReadyToWrite,
    WaitingForWrite,
    Dead,
}

impl DBusWriter {
    pub(crate) fn new(fd: i32, queue: &mut DBusQueue) -> Self {
        let current = queue.pop_front();

        Self {
            fd,
            current,
            state: State::ReadyToWrite,
        }
    }

    pub(crate) fn wants(&mut self, queue: &mut DBusQueue) -> Option<Wants> {
        match self.state {
            State::ReadyToWrite => {
                if self.current.is_none() {
                    self.current = queue.pop_front();
                }

                let buf = self.current.as_mut()?;

                self.state = State::WaitingForWrite;
                Some(Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                })
            }

            State::WaitingForWrite | State::Dead => None,
        }
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
        queue: &mut DBusQueue,
    ) -> Result<(), DBusError> {
        match (self.state, satisfy) {
            (State::Dead, _) => Ok(()),

            (State::WaitingForWrite, Satisfy::Write) => {
                if res < 0 {
                    return Err(DBusError::WriteError(format!("Write failed: {res}")));
                }
                let Some(message) = self.current.take() else {
                    return Err(DBusError::InternalError(
                        "malformed state: received Write, but there's no current message"
                            .to_string(),
                    ));
                };
                let bytes_written = res as usize;
                if bytes_written != message.len() {
                    return Err(DBusError::WriteError(format!(
                        "written is wrong: {bytes_written} vs {}",
                        message.len()
                    )));
                };

                if let Some(next) = queue.pop_front() {
                    self.current = Some(next);
                }
                self.state = State::ReadyToWrite;
                Ok(())
            }

            (state, satisfy) => Err(DBusError::InternalError(format!(
                "malformed state: {state:?} vs {satisfy:?}"
            ))),
        }
    }

    pub(crate) fn stop(&mut self) {
        self.state = State::Dead;
    }
}
