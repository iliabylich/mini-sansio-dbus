use anyhow::{Context as _, Result, ensure};
use mini_sansio_dbus::{
    DBusConnection, DBusError, DBusWants, EncodeError, IncomingMessage, IncomingValue, MessageType,
    OutgoingQueue,
    messages::org_freedesktop_dbus::{GetProperty, Hello},
    messaging::{DBusEncode, property::PropertyGet},
    value_is,
};
use rustix::{
    event::{PollFd, PollFlags},
    io::Errno,
};
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};

mod queue;
use queue::ExampleQueue;

struct PollDBus {
    conn: DBusConnection,
    fd: Option<OwnedFd>,
}

impl PollDBus {
    fn new() -> Result<Self> {
        let socket_path = std::env::var("DBUS_SYSTEM_BUS_ADDRESS")
            .ok()
            .and_then(|address| address.split_once('=').map(|(_, path)| path.to_string()))
            .unwrap_or_else(|| String::from("/var/run/dbus/system_bus_socket"));

        Ok(Self {
            conn: DBusConnection::new_system(&socket_path)?,
            fd: None,
        })
    }

    fn process_until_blocked_or_message_received<'a>(
        &'a mut self,
        queue: &mut ExampleQueue,
        readerbuf: &'a mut [u8],
    ) -> Result<ProcessResult<'a>> {
        loop {
            let wants = self
                .conn
                .wants(queue, readerbuf)?
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
                    rustix::net::connect(self.fd.as_ref().context("no FD")?, &addr)?;
                    log::info!("connect() succeeded");
                    self.conn.satisfy_connect()?;
                    return Ok(ProcessResult::Connected);
                }
                DBusWants::Read { buf, .. } => {
                    match rustix::io::read(self.fd.as_ref().context("no FD")?, buf) {
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
                                    self.fd.as_ref().context("no FD")?.as_fd(),
                                    PollFlags::IN,
                                )),
                            });
                        }
                        Err(err) => return Err(err.into()),
                    }
                }
                DBusWants::Write { buf, .. } => {
                    match rustix::io::write(self.fd.as_ref().context("no FD")?, buf) {
                        Ok(len) => {
                            self.conn.satisfy_write(len, queue)?;
                        }
                        Err(Errno::WOULDBLOCK) => {
                            return Ok(ProcessResult::ReadWrite {
                                message: None,
                                blocked_on: Some((
                                    self.fd.as_ref().context("no FD")?.as_fd(),
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

                    let write_res = rustix::io::write(self.fd.as_ref().context("no FD")?, writebuf);
                    let read_res = rustix::io::read(self.fd.as_ref().context("no FD")?, readbuf);

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
                                    Some((
                                        self.fd.as_ref().context("no FD")?.as_fd(),
                                        PollFlags::OUT,
                                    ))
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
                                        self.fd.as_ref().context("no FD")?.as_fd(),
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
    let mut primary_connection_id_buf = [0; 512];
    let mut queue = ExampleQueue::new();
    let mut readerbuf = [0; 1_024];

    queue.push_and_discard_reply::<Hello>(())?;

    let mut primary_connection_path_reply_handler = None;
    let mut primary_connection_id_reply_serial = 0;

    loop {
        match dbus.process_until_blocked_or_message_received(&mut queue, &mut readerbuf)? {
            ProcessResult::Connected => {
                primary_connection_path_reply_handler =
                    Some(queue.push_and_prepare_for_reply(GetPrimaryConnection, ())?);
            }
            ProcessResult::ReadWrite {
                message,
                blocked_on,
            } => {
                if let Some(message) = message {
                    let mut buf = String::new();
                    message.log(&mut buf)?;
                    eprintln!("{buf}");

                    if let Some(primary_connection_path_reply_handler) =
                        primary_connection_path_reply_handler.as_ref()
                        && let Some(primary_connection_path) =
                            primary_connection_path_reply_handler.handle(message)?
                    {
                        log::info!("Primary connection: {primary_connection_path}");

                        primary_connection_id_reply_serial = enqueue_get_property(
                            &mut queue,
                            &mut primary_connection_id_buf,
                            "org.freedesktop.NetworkManager",
                            &primary_connection_path,
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

struct GetPrimaryConnection;
impl DBusEncode for GetPrimaryConnection {
    type Data = ();

    fn encode((): Self::Data, buf: &mut [u8]) -> Result<&[u8], EncodeError> {
        GetProperty::encode(
            buf,
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager",
            "org.freedesktop.NetworkManager",
            "PrimaryConnection",
        )
    }
}
impl PropertyGet for GetPrimaryConnection {
    type Output = String;

    fn map(value: IncomingValue<'_>) -> Result<Self::Output, DBusError> {
        value_is!(value, IncomingValue::ObjectPath(value));
        Ok(value.to_string())
    }
}

fn enqueue_get_property(
    queue: &mut ExampleQueue,
    buf: &mut [u8],
    destination: &str,
    path: &str,
    interface: &str,
    property: &str,
) -> Result<u32> {
    let buf = GetProperty::encode(buf, destination, path, interface, property)?;
    let serial = queue.push_raw_buf(buf);
    Ok(serial)
}

fn poll(fd: BorrowedFd<'_>, events: PollFlags) -> Result<()> {
    let mut pollfds = [PollFd::new(&fd, events)];
    let ready = rustix::event::poll(&mut pollfds, None)?;
    ensure!(ready > 0);
    log::info!("poll() finished()");
    Ok(())
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
