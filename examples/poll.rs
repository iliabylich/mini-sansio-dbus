use anyhow::{Context as _, Result, ensure};
use mini_sansio_dbus::{
    DBusConnection, DBusError, DBusSerial, DBusWants, EncodeMessage, IncomingMessage,
    IncomingValue, MessageType, OutgoingQueue,
    messages::org_freedesktop_dbus::{GetProperty, Hello},
    value_is,
};
use rustix::{
    event::{PollFd, PollFlags},
    io::Errno,
};
use std::{
    collections::VecDeque,
    os::fd::{AsFd, BorrowedFd, OwnedFd},
};

#[derive(Debug, Default)]
struct ExampleQueue {
    messages: VecDeque<Vec<u8>>,
}

impl ExampleQueue {
    fn new() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }
}

impl OutgoingQueue for ExampleQueue {
    fn push(&mut self, message: &mut [u8], serial: u32) -> Result<(), DBusError> {
        DBusSerial::write_to_message(message, serial)?;
        self.messages.push_back(message.to_vec());
        Ok(())
    }

    fn front(&self) -> Option<&[u8]> {
        self.messages.front().map(Vec::as_slice)
    }

    fn pop_front(&mut self) {
        self.messages.pop_front();
    }
}

fn encode_and_queue<Q, B, M>(
    serial: &mut DBusSerial,
    queue: &mut Q,
    mut buf: B,
    message: &M,
) -> Result<u32, DBusError>
where
    Q: OutgoingQueue,
    B: AsMut<[u8]>,
    M: EncodeMessage,
{
    let next_serial = serial.current();
    let len = message.encode_message(buf.as_mut())?;
    let message = buf
        .as_mut()
        .get_mut(..len)
        .ok_or(DBusError::InternalError)?;
    queue.push(message, next_serial)?;
    serial.advance();
    Ok(next_serial)
}

struct PollDBus {
    conn: DBusConnection,
    fd: Option<OwnedFd>,
}

impl PollDBus {
    fn new() -> Result<Self> {
        Ok(Self {
            conn: DBusConnection::new_system()?,
            fd: None,
        })
    }

    fn process_until_blocked_or_message_received<'a>(
        &'a mut self,
        queue: &mut ExampleQueue,
        readerbuf: &'a mut Vec<u8>,
    ) -> Result<ProcessResult<'a>> {
        loop {
            let wants = self
                .conn
                .wants(queue, readerbuf)
                .context("DBus wants nothing")?;

            match wants {
                DBusWants::Socket { domain, r#type, .. } => {
                    log::info!("starting socket()");
                    let fd = rustix::net::socket(domain, r#type, None)?;
                    self.fd = Some(fd);
                    log::info!("socket() succeeded");
                    self.conn.satisfy_socket()?;
                }
                DBusWants::Connect { addr, .. } => {
                    log::info!("starting connect()");
                    rustix::net::connect(self.fd.as_ref().expect("no FD"), &addr)?;
                    log::info!("connect() succeeded");
                    self.conn.satisfy_connect()?;
                    return Ok(ProcessResult::Connected);
                }
                DBusWants::Read { buf, .. } => {
                    match rustix::io::read(self.fd.as_ref().expect("no FD"), buf) {
                        Ok(len) => {
                            let message = self.conn.satisfy_read(len, readerbuf)?;
                            return Ok(ProcessResult::ReadWrite {
                                message,
                                blocked_on: None,
                            });
                        }
                        Err(Errno::WOULDBLOCK) => {
                            return Ok(ProcessResult::ReadWrite {
                                message: None,
                                blocked_on: Some((
                                    self.fd.as_ref().expect("no FD").as_fd(),
                                    PollFlags::IN,
                                )),
                            });
                        }
                        Err(err) => return Err(err.into()),
                    }
                }
                DBusWants::Write { buf, .. } => {
                    match rustix::io::write(self.fd.as_ref().expect("no FD"), buf) {
                        Ok(len) => {
                            self.conn.satisfy_write(len, queue)?;
                        }
                        Err(Errno::WOULDBLOCK) => {
                            return Ok(ProcessResult::ReadWrite {
                                message: None,
                                blocked_on: Some((
                                    self.fd.as_ref().expect("no FD").as_fd(),
                                    PollFlags::OUT,
                                )),
                            });
                        }
                        Err(err) => return Err(err.into()),
                    }
                }
                DBusWants::ReadWrite {
                    readbuf, writebuf, ..
                } => {
                    let mut blocked_on_write = false;

                    let write_res = rustix::io::write(self.fd.as_ref().expect("no FD"), writebuf);
                    let read_res = rustix::io::read(self.fd.as_ref().expect("no FD"), readbuf);

                    match write_res {
                        Ok(len) => {
                            self.conn.satisfy_write(len, queue)?;
                        }
                        Err(Errno::WOULDBLOCK) => blocked_on_write = true,
                        Err(err) => return Err(err.into()),
                    }

                    match read_res {
                        Ok(len) => {
                            let message = self.conn.satisfy_read(len, readerbuf)?;

                            return Ok(ProcessResult::ReadWrite {
                                message,
                                blocked_on: if blocked_on_write {
                                    Some((self.fd.as_ref().expect("no FD").as_fd(), PollFlags::OUT))
                                } else {
                                    None
                                },
                            });
                        }
                        Err(Errno::WOULDBLOCK) => {
                            return Ok(ProcessResult::ReadWrite {
                                message: None,
                                blocked_on: if blocked_on_write {
                                    Some((
                                        self.fd.as_ref().expect("no FD").as_fd(),
                                        PollFlags::IN | PollFlags::OUT,
                                    ))
                                } else {
                                    None
                                },
                            });
                        }
                        Err(err) => return Err(err.into()),
                    }
                }
            }
        }
    }
}

enum ProcessResult<'a> {
    Connected,
    ReadWrite {
        message: Option<IncomingMessage<'a>>,
        blocked_on: Option<(BorrowedFd<'a>, PollFlags)>,
    },
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut dbus = PollDBus::new()?;
    let mut serial = DBusSerial::new();
    let primary_connection_path_buf = [0; 512];
    let primary_connection_id_buf = [0; 512];
    let mut queue = ExampleQueue::new();
    let mut readerbuf = vec![];
    {
        let mut buf = Hello::ENCODED;
        queue.push(&mut buf, serial.current())?;
        serial.advance();
    }

    let mut primary_connection_path_reply_serial = 0;
    let mut primary_connection_id_reply_serial = 0;

    loop {
        match dbus.process_until_blocked_or_message_received(&mut queue, &mut readerbuf)? {
            ProcessResult::Connected => {
                primary_connection_path_reply_serial = enqueue_get_property(
                    &mut serial,
                    &mut queue,
                    primary_connection_path_buf,
                    "org.freedesktop.NetworkManager",
                    "/org/freedesktop/NetworkManager",
                    "org.freedesktop.NetworkManager",
                    "PrimaryConnection",
                )?;
            }
            ProcessResult::ReadWrite {
                message,
                blocked_on,
            } => {
                if let Some(message) = message {
                    let mut buf = String::new();
                    message.log(&mut buf)?;
                    eprintln!("{buf}");

                    if let Some(primary_connection) = try_parse_primary_connection_path_reply(
                        message,
                        primary_connection_path_reply_serial,
                    )? {
                        log::info!("Primary connection: {primary_connection}");

                        primary_connection_id_reply_serial = enqueue_get_property(
                            &mut serial,
                            &mut queue,
                            primary_connection_id_buf,
                            "org.freedesktop.NetworkManager",
                            primary_connection,
                            "org.freedesktop.NetworkManager.Connection.Active",
                            "Id",
                        )?;
                    }

                    if let Some(id) = try_parse_primary_connection_id_reply(
                        message,
                        primary_connection_id_reply_serial,
                    )? {
                        log::info!("Primary connection ID: {id}");
                        break;
                    }
                }

                if let Some((fd, events)) = blocked_on {
                    poll(fd, events)?;
                }
            }
        }
    }

    Ok(())
}

fn enqueue_get_property(
    serial: &mut DBusSerial,
    queue: &mut ExampleQueue,
    buf: [u8; 512],
    destination: &str,
    path: &str,
    interface: &str,
    property: &str,
) -> Result<u32> {
    encode_and_queue(
        serial,
        queue,
        buf,
        &GetProperty::new(destination, path, interface, property),
    )
    .map_err(Into::into)
}

fn poll(fd: BorrowedFd<'_>, events: PollFlags) -> Result<()> {
    let mut pollfds = [PollFd::new(&fd, events)];
    let ready = rustix::event::poll(&mut pollfds, None)?;
    ensure!(ready > 0);
    log::info!("poll() finished()");
    Ok(())
}

fn try_parse_primary_connection_path_reply<'a>(
    message: IncomingMessage<'a>,
    reply_serial: u32,
) -> Result<Option<&'a str>> {
    if message.message_type != MessageType::MethodReturn
        || message.reply_serial != Some(reply_serial)
    {
        return Ok(None);
    }

    let mut body = message.body.context("no body")?;
    let path = body.try_next()?.context("empty body")?;
    value_is!(path, IncomingValue::Variant(path));
    let path = path.materialize()?;
    value_is!(path, IncomingValue::ObjectPath(path));
    Ok(Some(path))
}

fn try_parse_primary_connection_id_reply<'a>(
    message: IncomingMessage<'a>,
    reply_serial: u32,
) -> Result<Option<&'a str>> {
    if message.message_type != MessageType::MethodReturn
        || message.reply_serial != Some(reply_serial)
    {
        return Ok(None);
    }

    let mut body = message.body.context("no body")?;
    let id = body.try_next()?.context("empty body")?;
    value_is!(id, IncomingValue::Variant(id));
    let id = id.materialize()?;
    value_is!(id, IncomingValue::String(path));
    Ok(Some(path))
}
