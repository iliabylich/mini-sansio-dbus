use anyhow::{Context, Result, bail};
use mini_sansio_dbus::{DBusConnection, DBusQueue, DBusWants, IncomingMessage, MessageType};
use std::os::fd::OwnedFd;

struct BlockingDBus {
    conn: DBusConnection,
    fd: Option<OwnedFd>,
}

impl BlockingDBus {
    fn new() -> Result<Self> {
        let conn = DBusConnection::new_session()?;
        Ok(Self { conn, fd: None })
    }

    fn socket(&mut self, queue: &mut DBusQueue, readerbuf: &mut Vec<u8>) -> Result<()> {
        log::info!("Getting a socket...");
        let wants = self
            .conn
            .wants(queue, readerbuf)
            .context("expected socket, got None")?;
        let DBusWants::Socket { domain, r#type, .. } = wants else {
            bail!("at first there must be connect, bug?");
        };
        let fd = rustix::net::socket(domain, r#type, None)?;
        self.conn.satisfy_socket()?;

        log::info!("socket() returned {fd:?}");
        self.fd = Some(fd);

        Ok(())
    }

    fn connect(&mut self, queue: &mut DBusQueue, readerbuf: &mut Vec<u8>) -> Result<()> {
        log::info!("Connecting...");
        let wants = self
            .conn
            .wants(queue, readerbuf)
            .context("expected connect, got None")?;
        let DBusWants::Connect { addr, .. } = wants else {
            bail!("at first there must be connect, bug?");
        };
        rustix::net::connect(self.fd.as_ref().expect("no FD"), &addr)?;
        self.conn.satisfy_connect()?;

        log::info!("connect() succeeded");

        Ok(())
    }

    fn read_write<'a>(
        &mut self,
        queue: &mut DBusQueue,
        readerbuf: &'a mut Vec<u8>,
    ) -> Result<Option<IncomingMessage<'a>>> {
        let wants = self.conn.wants(queue, readerbuf).context("wants nothing")?;
        log::info!("<< {wants:?}");

        match wants {
            DBusWants::Write { buf, .. } => {
                let len = rustix::io::write(self.fd.as_ref().expect("no FD"), buf)?;
                self.conn.satisfy_write(len, queue)?;
                Ok(None)
            }
            DBusWants::Read { buf, .. } => {
                let len = rustix::io::read(self.fd.as_ref().expect("no FD"), buf)?;
                self.conn.satisfy_read(len, readerbuf).map_err(Into::into)
            }
            DBusWants::ReadWrite {
                readbuf, writebuf, ..
            } => {
                let writelen = rustix::io::write(self.fd.as_ref().expect("no FD"), writebuf)?;
                let readlen = rustix::io::read(self.fd.as_ref().expect("no FD"), readbuf)?;

                self.conn.satisfy_write(writelen, queue)?;
                self.conn
                    .satisfy_read(readlen, readerbuf)
                    .map_err(Into::into)
            }
            _ => unreachable!("wants {wants:?}"),
        }
    }
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut dbus = BlockingDBus::new()?;
    let mut queue = DBusQueue::new();
    let mut readerbuf = vec![];

    dbus.socket(&mut queue, &mut readerbuf)?;
    dbus.connect(&mut queue, &mut readerbuf)?;

    loop {
        if let Some(message) = dbus.read_write(&mut queue, &mut readerbuf)? {
            log::info!("Recived");
            message.log()?;

            if message.message_type == MessageType::Signal
                && message
                    .member
                    .is_some_and(|member| member == "NameAcquired")
            {
                log::info!("Connected to DBus!");
                break;
            }
        }
    }

    Ok(())
}
