use anyhow::{Context as _, Result, ensure};
use mini_sansio_dbus::{
    Conf, DBusConnection, DBusError, DBusWants, IncomingMessage, IncomingValue,
    messages::org_freedesktop_dbus::Hello, messaging::property::Property, value_is,
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
    let mut queue = ExampleQueue::new();
    let mut readerbuf = [0; 1_024];

    queue.push_and_discard_reply::<Hello>(())?;

    let mut primary_connection_path_reply_handler = None;
    let mut primary_connection_id_reply_handler = None;

    loop {
        match dbus.process_until_blocked_or_message_received(&mut queue, &mut readerbuf)? {
            ProcessResult::Connected => {
                primary_connection_path_reply_handler =
                    Some(PrimaryConnection.get(&mut [0; 1_024], &mut queue)?);
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

                        let conn_id = ConnId {
                            conn_path: primary_connection_path,
                        };
                        primary_connection_id_reply_handler =
                            Some(conn_id.get(&mut [0; 1_024], &mut queue)?);
                    }

                    if let Some(primary_connection_id_reply_handler) =
                        primary_connection_id_reply_handler.as_ref()
                        && let Some(id) = primary_connection_id_reply_handler.handle(message)?
                    {
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

#[derive(Clone)]
struct PrimaryConnection;
impl Property for PrimaryConnection {
    type Output = String;

    const DESTINATION: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager");
    const PATH: Conf<str, Self> = Conf::constant("/org/freedesktop/NetworkManager");
    const INTERFACE: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager");
    const PROPERTY_NAME: Conf<str, Self> = Conf::constant("PrimaryConnection");

    fn map(value: IncomingValue<'_>) -> Result<Self::Output, DBusError> {
        value_is!(value, IncomingValue::ObjectPath(value));
        Ok(value.to_string())
    }
}

#[derive(Clone)]
struct ConnId {
    conn_path: String,
}
impl Property for ConnId {
    type Output = String;

    const DESTINATION: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager");
    const PATH: Conf<str, Self> = Conf::dynamic(|this| this.conn_path.as_str());
    const INTERFACE: Conf<str, Self> =
        Conf::constant("org.freedesktop.NetworkManager.Connection.Active");
    const PROPERTY_NAME: Conf<str, Self> = Conf::constant("Id");

    fn map(value: IncomingValue<'_>) -> Result<Self::Output, DBusError> {
        value_is!(value, IncomingValue::String(value));
        Ok(value.to_string())
    }
}

fn poll(fd: BorrowedFd<'_>, events: PollFlags) -> Result<()> {
    let mut pollfds = [PollFd::new(&fd, events)];
    let ready = rustix::event::poll(&mut pollfds, None)?;
    ensure!(ready > 0);
    log::info!("poll() finished()");
    Ok(())
}
