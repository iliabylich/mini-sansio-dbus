use anyhow::{Context as _, Result, bail, ensure};
use mini_sansio_dbus::{
    DBusConnection, DBusQueue, DBusSatisfy, DBusWants, IncomingMessage, IncomingValue, MessageType,
    messages::org_freedesktop_dbus::GetProperty, value_is,
};

struct PollDBus {
    conn: DBusConnection,
    fd: Option<i32>,
}

impl PollDBus {
    fn new() -> Self {
        Self {
            conn: DBusConnection::new_system(),
            fd: None,
        }
    }

    fn process_until_blocked_or_message_received<'a>(
        &mut self,
        queue: &mut DBusQueue,
        readerbuf: &'a mut Vec<u8>,
    ) -> Result<ProcessResult<'a>> {
        loop {
            let wants = self
                .conn
                .wants(queue, readerbuf)
                .context("DBus wants nothing")?;

            match wants {
                DBusWants::Socket { domain, r#type, .. } => {
                    self.socket(domain, r#type, queue, readerbuf)?;
                }
                DBusWants::Connect {
                    fd, addr, addrlen, ..
                } => {
                    self.connect(fd, addr, addrlen, queue, readerbuf)?;
                    return Ok(ProcessResult::Connected);
                }
                DBusWants::Read { fd, buf, len, .. } => {
                    match self.read(fd, buf, len, queue, readerbuf)? {
                        ReadResult::Ok(message) => {
                            return Ok(ProcessResult::ReadWrite {
                                message,
                                blocked_on: None,
                            });
                        }
                        ReadResult::Blocked => {
                            return Ok(ProcessResult::ReadWrite {
                                message: None,
                                blocked_on: Some((fd, libc::POLLIN)),
                            });
                        }
                    }
                }
                DBusWants::Write { fd, buf, len, .. } => {
                    match self.write(fd, buf, len, queue, readerbuf)? {
                        WriteResult::Ok => {}
                        WriteResult::Blocked => {
                            return Ok(ProcessResult::ReadWrite {
                                message: None,
                                blocked_on: Some((fd, libc::POLLOUT)),
                            });
                        }
                    }
                }
                DBusWants::ReadWrite {
                    fd,
                    readbuf,
                    readlen,
                    writebuf,
                    writelen,
                    ..
                } => {
                    let mut blocked_on_write = false;

                    match self.write(fd, writebuf, writelen, queue, readerbuf)? {
                        WriteResult::Ok => {}
                        WriteResult::Blocked => blocked_on_write = true,
                    }

                    match self.read(fd, readbuf, readlen, queue, readerbuf)? {
                        ReadResult::Ok(message) => {
                            return Ok(ProcessResult::ReadWrite {
                                message,
                                blocked_on: if blocked_on_write {
                                    Some((fd, libc::POLLOUT))
                                } else {
                                    None
                                },
                            });
                        }
                        ReadResult::Blocked => {
                            return Ok(ProcessResult::ReadWrite {
                                message: None,
                                blocked_on: if blocked_on_write {
                                    Some((fd, libc::POLLIN | libc::POLLOUT))
                                } else {
                                    None
                                },
                            });
                        }
                    }
                }
            }
        }
    }

    fn socket(
        &mut self,
        domain: i32,
        r#type: i32,
        queue: &mut DBusQueue,
        readerbuf: &mut Vec<u8>,
    ) -> Result<()> {
        let res = unsafe { libc::socket(domain, r#type, 0) };
        log::info!("socket() returned {res}");
        let None = self
            .conn
            .satisfy(DBusSatisfy::Socket, res, readerbuf, queue)?
        else {
            bail!("expected None");
        };
        Ok(())
    }

    fn connect(
        &mut self,
        fd: i32,
        addr: *const libc::sockaddr,
        addrlen: u32,
        queue: &mut DBusQueue,
        readerbuf: &mut Vec<u8>,
    ) -> Result<()> {
        self.fd = Some(fd);

        let res = unsafe { libc::connect(fd, addr, addrlen) };
        log::info!("connect() returned {res}");
        let None = self
            .conn
            .satisfy(DBusSatisfy::Connect, res, readerbuf, queue)?
        else {
            bail!("expected None");
        };

        set_nonblocking(fd)?;

        Ok(())
    }

    fn read<'a>(
        &mut self,
        fd: i32,
        buf: *mut u8,
        len: usize,
        queue: &mut DBusQueue,
        readerbuf: &'a mut Vec<u8>,
    ) -> Result<ReadResult<'a>> {
        log::info!("read: from {fd} into {buf:?} {len} bytes");
        let res = unsafe { libc::read(fd, buf.cast(), len) } as i32;
        log::info!("read: Got {res}");

        if res < 0 {
            log::info!("read: errno is {}", errno());
            if errno() == libc::EINTR || errno() == libc::EAGAIN {
                Ok(ReadResult::Blocked)
            } else {
                Err(anyhow::anyhow!("read failed: {}", errno()))
            }
        } else {
            match self.conn.satisfy(DBusSatisfy::Read, res, readerbuf, queue) {
                Ok(message) => Ok(ReadResult::Ok(message)),
                Err(err) => Err(anyhow::anyhow!(err)),
            }
        }
    }

    fn write(
        &mut self,
        fd: i32,
        buf: *const u8,
        len: usize,
        queue: &mut DBusQueue,
        readerbuf: &mut Vec<u8>,
    ) -> Result<WriteResult> {
        log::info!("write: to {fd} from {buf:?} {len} bytes");
        let res = unsafe { libc::write(fd, buf.cast(), len) } as i32;
        log::info!("write: {res}");

        if res < 0 {
            log::info!("read: errno is {}", errno());
            if errno() == libc::EINTR || errno() == libc::EAGAIN {
                Ok(WriteResult::Blocked)
            } else {
                Err(anyhow::anyhow!("write failed: {res}"))
            }
        } else {
            match self.conn.satisfy(DBusSatisfy::Write, res, readerbuf, queue) {
                Ok(None) => Ok(WriteResult::Ok),
                Ok(Some(_)) => Err(anyhow::anyhow!("write never returns a message")),
                Err(err) => Err(anyhow::anyhow!(err)),
            }
        }
    }
}

impl Drop for PollDBus {
    fn drop(&mut self) {
        if let Some(fd) = self.fd {
            unsafe {
                libc::close(fd);
            }
        }
    }
}

enum ReadResult<'a> {
    Ok(Option<IncomingMessage<'a>>),
    Blocked,
}

enum WriteResult {
    Ok,
    Blocked,
}

enum ProcessResult<'a> {
    Connected,
    ReadWrite {
        message: Option<IncomingMessage<'a>>,
        blocked_on: Option<(i32, i16)>,
    },
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut dbus = PollDBus::new();
    let mut queue = DBusQueue::new();
    let mut readerbuf = vec![];

    let mut primary_connection_path_reply_serial = 0;
    let mut primary_connection_id_reply_serial = 0;

    loop {
        match dbus.process_until_blocked_or_message_received(&mut queue, &mut readerbuf)? {
            ProcessResult::Connected => {
                primary_connection_path_reply_serial = queue.push_back(GetProperty::build(
                    "org.freedesktop.NetworkManager",
                    "/org/freedesktop/NetworkManager",
                    "org.freedesktop.NetworkManager",
                    "PrimaryConnection",
                ));
            }
            ProcessResult::ReadWrite {
                message,
                blocked_on,
            } => {
                if let Some(message) = message {
                    message.log()?;

                    if let Some(primary_connection) = try_parse_primary_connection_path_reply(
                        message,
                        primary_connection_path_reply_serial,
                    )? {
                        log::info!("Primary connection: {primary_connection}");

                        primary_connection_id_reply_serial = queue.push_back(GetProperty::build(
                            "org.freedesktop.NetworkManager",
                            primary_connection,
                            "org.freedesktop.NetworkManager.Connection.Active",
                            "Id",
                        ));
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

fn set_nonblocking(fd: i32) -> Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL, 0) };
    ensure!(flags != -1);
    let res = unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    ensure!(res != -1);
    Ok(())
}

fn errno() -> i32 {
    std::io::Error::last_os_error().raw_os_error().unwrap()
}

fn poll(fd: i32, events: i16) -> Result<()> {
    let mut pollfd = libc::pollfd {
        fd,
        events,
        revents: 0,
    };
    log::info!(
        "poll({fd}, POLLIN={} POLLOUT={})",
        events & libc::POLLIN != 0,
        events & libc::POLLOUT != 0,
    );
    let ready = unsafe { libc::poll(&raw mut pollfd, 1, -1) };
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
