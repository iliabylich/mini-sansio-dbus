use anyhow::{Context, Result, bail};
use libc::{close, connect, read, socket, write};
use mini_sansio_dbus::{DBusConnection, DBusQueue, MessageType, Satisfy, Wants};

fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut conn = DBusConnection::new_session()?;
    let mut queue = DBusQueue::new();
    let mut readerbuf = vec![];

    // socket
    log::info!("Getting a socket...");
    let wants = conn
        .wants(&mut queue, &mut readerbuf)
        .context("expected connect, got None")?;
    let Wants::Socket { domain, r#type } = wants else {
        bail!("at first there must be connect, bug?");
    };
    let res = unsafe { socket(domain, r#type, 0) };
    log::info!("socket() returned {res}");
    let None = conn.satisfy(Satisfy::Socket, res, &mut readerbuf, &mut queue)? else {
        bail!("expected None");
    };

    // connect
    log::info!("Connecting...");
    let wants = conn
        .wants(&mut queue, &mut readerbuf)
        .context("expected connect, got None")?;
    let Wants::Connect { fd, addr, addrlen } = wants else {
        bail!("at first there must be connect, bug?");
    };
    let res = unsafe { connect(fd, addr, addrlen) };
    log::info!("connect() returned {res}");
    let None = conn.satisfy(Satisfy::Connect, res, &mut readerbuf, &mut queue)? else {
        bail!("expected None");
    };

    // read/write loop
    loop {
        let wants = conn
            .wants(&mut queue, &mut readerbuf)
            .context("expected connect, got None")?;
        log::info!("<< {wants:?}");

        let out = match wants {
            Wants::Write { fd, buf, len } => {
                let res = unsafe { write(fd, buf.cast(), len) };
                log::info!(">> written {res}");
                let None = conn.satisfy(Satisfy::Write, res as i32, &mut readerbuf, &mut queue)?
                else {
                    bail!("write never returns a message");
                };
                None
            }
            Wants::Read { fd, buf, len } => {
                let res = unsafe { read(fd, buf.cast(), len) };
                log::info!(">> read {res}");
                conn.satisfy(Satisfy::Read, res as i32, &mut readerbuf, &mut queue)?
            }
            Wants::ReadWrite {
                fd,
                readbuf,
                readlen,
                writebuf,
                writelen,
            } => {
                let res = unsafe { write(fd, writebuf.cast(), writelen) };
                log::info!(">> written {res}");
                let None = conn.satisfy(Satisfy::Write, res as i32, &mut readerbuf, &mut queue)?
                else {
                    bail!("write never returns a message");
                };

                let res = unsafe { read(fd, readbuf.cast(), readlen) };
                log::info!(">> read {res}");
                conn.satisfy(Satisfy::Read, res as i32, &mut readerbuf, &mut queue)?
            }
            _ => unreachable!(),
        };

        if let Some(message) = out {
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

    unsafe { close(fd) };

    Ok(())
}
