use anyhow::{Context, Result, bail, ensure};
use mini_sansio_dbus::{
    DBusConnection, DBusQueue, DBusSatisfy, DBusWants, IncomingMessage, MessageType,
};

struct BlockingDBus {
    conn: DBusConnection,
    fd: Option<i32>,
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
            .context("expected connect, got None")?;
        let DBusWants::Socket { domain, r#type, .. } = wants else {
            bail!("at first there must be connect, bug?");
        };
        let res = unsafe { libc::socket(domain, r#type, 0) };
        log::info!("socket() returned {res}");
        ensure!(
            self.conn
                .satisfy(DBusSatisfy::Socket, res, readerbuf, queue)?
                .is_none(),
            "expected None"
        );
        Ok(())
    }

    fn connect(&mut self, queue: &mut DBusQueue, readerbuf: &mut Vec<u8>) -> Result<()> {
        log::info!("Connecting...");
        let wants = self
            .conn
            .wants(queue, readerbuf)
            .context("expected connect, got None")?;
        let DBusWants::Connect {
            fd, addr, addrlen, ..
        } = wants
        else {
            bail!("at first there must be connect, bug?");
        };
        let res = unsafe { libc::connect(fd, addr, addrlen) };
        log::info!("connect() returned {res}");
        ensure!(
            self.conn
                .satisfy(DBusSatisfy::Connect, res, readerbuf, queue)?
                .is_none(),
            "expected None"
        );

        self.fd = Some(fd);
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
            DBusWants::Write { fd, buf, len, .. } => {
                self.write(fd, buf, len, queue, readerbuf)?;
                Ok(None)
            }
            DBusWants::Read { fd, buf, len, .. } => self.read(fd, buf, len, queue, readerbuf),
            DBusWants::ReadWrite {
                fd,
                readbuf,
                readlen,
                writebuf,
                writelen,
                ..
            } => {
                self.write(fd, writebuf, writelen, queue, readerbuf)?;
                self.read(fd, readbuf, readlen, queue, readerbuf)
            }
            _ => unreachable!("wants {wants:?}"),
        }
    }

    fn read<'a>(
        &mut self,
        fd: i32,
        buf: *mut u8,
        len: usize,
        queue: &mut DBusQueue,
        readerbuf: &'a mut Vec<u8>,
    ) -> Result<Option<IncomingMessage<'a>>> {
        let res = unsafe { libc::read(fd, buf.cast(), len) } as i32;
        log::info!(">> written {res}");
        let out = self
            .conn
            .satisfy(DBusSatisfy::Read, res, readerbuf, queue)?;
        Ok(out)
    }

    fn write(
        &mut self,
        fd: i32,
        buf: *const u8,
        len: usize,
        queue: &mut DBusQueue,
        readerbuf: &mut Vec<u8>,
    ) -> Result<()> {
        let res = unsafe { libc::write(fd, buf.cast(), len) } as i32;
        log::info!(">> written {res}");
        ensure!(
            self.conn
                .satisfy(DBusSatisfy::Write, res as i32, readerbuf, queue)?
                .is_none(),
            "write never returns a message"
        );
        Ok(())
    }
}

impl Drop for BlockingDBus {
    fn drop(&mut self) {
        if let Some(fd) = self.fd {
            unsafe {
                libc::close(fd);
            }
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
