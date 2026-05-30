use anyhow::{Context, Result, bail};
use mini_sansio_dbus::DBusSerial;
use mini_sansio_dbus::{
    DBusConnection, DBusWants, IncomingMessage, MessageType, OutgoingQueue,
    messages::org_freedesktop_dbus::Hello, messaging::DBusSend as _,
};
use std::{collections::VecDeque, os::fd::OwnedFd};

#[derive(Debug)]
struct ExampleQueue {
    serial: DBusSerial,
    messages: VecDeque<Vec<u8>>,
}

impl ExampleQueue {
    fn new() -> Self {
        Self {
            serial: DBusSerial::new(),
            messages: VecDeque::new(),
        }
    }
}

impl ExampleQueue {
    fn next_serial(&mut self) -> u32 {
        let serial = self.serial.current();
        self.serial.advance();
        serial
    }
}

impl OutgoingQueue<'_> for ExampleQueue {
    fn push(&mut self, message: &[u8]) -> u32 {
        let serial = self.next_serial();
        let mut message = message.to_vec();
        DBusSerial::write_to_message(&mut message, serial).unwrap();
        self.messages.push_back(message);
        serial
    }

    fn peek(&self) -> Option<&[u8]> {
        self.messages.front().map(|m| m.as_slice())
    }

    fn pop(&mut self) {
        self.messages.pop_front();
    }
}

struct BlockingDBus {
    conn: DBusConnection,
    fd: Option<OwnedFd>,
}

impl BlockingDBus {
    fn new() -> Result<Self> {
        let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")?;
        let (_, address) = address
            .split_once('=')
            .context("malformed $DBUS_SESSION_BUS_ADDRESS")?;
        let conn = DBusConnection::new_session(address)?;
        Ok(Self { conn, fd: None })
    }

    fn socket(&mut self, queue: &ExampleQueue, readerbuf: &mut [u8]) -> Result<()> {
        log::info!("Getting a socket...");
        let wants = self
            .conn
            .wants(queue, readerbuf)?
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

    fn connect(&mut self, queue: &ExampleQueue, readerbuf: &mut [u8]) -> Result<()> {
        log::info!("Connecting...");
        let wants = self
            .conn
            .wants(queue, readerbuf)?
            .context("expected connect, got None")?;
        let DBusWants::Connect { addr, .. } = wants else {
            bail!("at first there must be connect, bug?");
        };
        rustix::net::connect(self.fd.as_ref().context("no FD")?, &addr)?;
        self.conn.satisfy_connect()?;

        log::info!("connect() succeeded");

        Ok(())
    }

    fn read_write<'a>(
        &mut self,
        queue: &mut ExampleQueue,
        readerbuf: &'a mut [u8],
    ) -> Result<Option<IncomingMessage<'a>>> {
        let wants = self
            .conn
            .wants(queue, readerbuf)?
            .context("wants nothing")?;
        log::info!("<< {wants:?}");

        match wants {
            DBusWants::Write { buf, .. } => {
                let len = rustix::io::write(self.fd.as_ref().context("no FD")?, buf)?;
                self.conn.satisfy_write(len, queue)?;
                Ok(None)
            }
            DBusWants::Read { buf, .. } => {
                let len = rustix::io::read(self.fd.as_ref().context("no FD")?, buf)?;
                self.conn.satisfy_read(len, readerbuf).map_err(Into::into)
            }
            DBusWants::ReadWrite {
                readbuf, writebuf, ..
            } => {
                let writelen = rustix::io::write(self.fd.as_ref().context("no FD")?, writebuf)?;
                let readlen = rustix::io::read(self.fd.as_ref().context("no FD")?, readbuf)?;

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
    let mut queue = ExampleQueue::new();
    let mut readerbuf = [0; 1_024];

    let Ok(_) = Hello::send(&mut queue);

    dbus.socket(&queue, &mut readerbuf)?;
    dbus.connect(&queue, &mut readerbuf)?;

    loop {
        if let Some(message) = dbus.read_write(&mut queue, &mut readerbuf)? {
            log::info!("Received");
            let mut buf = String::new();
            message.log(&mut buf)?;
            eprintln!("{buf}");

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
