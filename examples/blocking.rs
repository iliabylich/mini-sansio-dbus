use anyhow::{Context, Result, bail};
use mini_sansio_dbus::{
    DBusConnection, DBusError, DBusSerial, DBusWants, EncodeMessage, IncomingMessage, MessageType,
    OutgoingQueue, messages::org_freedesktop_dbus::Hello,
};
use std::{collections::VecDeque, os::fd::OwnedFd};

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

struct BlockingDBus {
    conn: DBusConnection,
    fd: Option<OwnedFd>,
}

impl BlockingDBus {
    fn new() -> Result<Self> {
        let conn = DBusConnection::new_session()?;
        Ok(Self { conn, fd: None })
    }

    fn socket(&mut self, queue: &ExampleQueue, readerbuf: &mut Vec<u8>) -> Result<()> {
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

    fn connect(&mut self, queue: &ExampleQueue, readerbuf: &mut Vec<u8>) -> Result<()> {
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
        queue: &mut ExampleQueue,
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
    let mut serial = DBusSerial::new();
    let hello_buf = [0; 256];
    let mut queue = ExampleQueue::new();
    let mut readerbuf = vec![];
    encode_and_queue(&mut serial, &mut queue, hello_buf, &Hello)?;

    dbus.socket(&queue, &mut readerbuf)?;
    dbus.connect(&queue, &mut readerbuf)?;

    loop {
        if let Some(message) = dbus.read_write(&mut queue, &mut readerbuf)? {
            log::info!("Recived");
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
