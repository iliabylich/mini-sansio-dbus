use anyhow::{Context, Result, bail};
use mini_sansio_dbus::{
    DBusConnection, DBusError, DBusSerial, DBusWants, EncodeMessage, EncodedMessage,
    IncomingMessage, MessageType, OutgoingQueue, messages::org_freedesktop_dbus::Hello,
};
use std::{collections::VecDeque, os::fd::OwnedFd};

#[derive(Debug, Default)]
struct ExampleQueue<M> {
    messages: VecDeque<M>,
}

impl<M> ExampleQueue<M> {
    fn new() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }
}

impl<M: AsRef<[u8]>> OutgoingQueue for ExampleQueue<M> {
    type Message = M;
    type Error = core::convert::Infallible;

    fn push(&mut self, message: Self::Message) -> Result<(), Self::Error> {
        self.messages.push_back(message);
        Ok(())
    }

    fn front(&self) -> Option<&[u8]> {
        self.messages.front().map(AsRef::as_ref)
    }

    fn pop_front(&mut self) -> Option<Self::Message> {
        self.messages.pop_front()
    }
}

fn encode_and_queue<Q, B, M>(
    serial: &mut DBusSerial,
    queue: &mut Q,
    mut buf: B,
    message: &M,
) -> Result<u32, DBusError>
where
    Q: OutgoingQueue<Message = EncodedMessage<B>>,
    B: AsMut<[u8]> + AsRef<[u8]>,
    M: EncodeMessage,
{
    let next_serial = serial.current();
    let len = message.encode_message(buf.as_mut())?;
    let mut message = EncodedMessage::new(buf, len);
    message.set_serial(next_serial)?;
    queue
        .push(message)
        .map_err(|_| DBusError::OutgoingQueueRejected)?;
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

    fn socket(
        &mut self,
        queue: &ExampleQueue<EncodedMessage<[u8; 256]>>,
        readerbuf: &mut Vec<u8>,
    ) -> Result<()> {
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

    fn connect(
        &mut self,
        queue: &ExampleQueue<EncodedMessage<[u8; 256]>>,
        readerbuf: &mut Vec<u8>,
    ) -> Result<()> {
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
        queue: &mut ExampleQueue<EncodedMessage<[u8; 256]>>,
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
    let mut queue = ExampleQueue::<EncodedMessage<[u8; 256]>>::new();
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
