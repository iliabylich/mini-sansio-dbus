use anyhow::{Context, Result};
use mini_sansio_dbus::{
    DBusConnection, DBusConnector, DBusConnectorWants, MessageType,
    messages::org_freedesktop_dbus::Hello,
};
use rustix::net::{AddressFamily, SocketAddrUnix, SocketType};

mod queue;
use queue::ExampleQueue;

fn main() -> Result<()> {
    let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")?;
    let (_, address) = address
        .split_once('=')
        .context("malformed $DBUS_SESSION_BUS_ADDRESS")?;
    let address = SocketAddrUnix::new(address)?;

    println!("socket()");
    let fd = rustix::net::socket(AddressFamily::UNIX, SocketType::STREAM, None)?;
    println!("connect()");
    rustix::net::connect(&fd, &address)?;

    let mut readbuf = [0; 1_024];

    let mut connector = DBusConnector::new();
    let seq = loop {
        let wants = connector.wants(&mut readbuf)?;
        match wants {
            DBusConnectorWants::Read { buf, .. } => {
                println!("read({})", buf.len());
                let len = rustix::io::read(&fd, buf)?;
                connector.satisfy_read(len, &mut readbuf)?;
            }
            DBusConnectorWants::Write { buf, .. } => {
                println!("write({})", buf.len());
                let len = rustix::io::write(&fd, buf)?;
                if let Some(seq) = connector.satisfy_write(len)? {
                    break seq;
                }
            }
        }
    };
    println!("Connected!");
    drop(connector);

    let mut queue = ExampleQueue::new();
    queue.push_and_discard_reply::<Hello>(())?;

    let mut dbus = DBusConnection::new(seq);
    loop {
        let (read, write) = dbus.wants(&mut queue, &mut readbuf)?;

        if let Some(write) = write {
            println!("write({})", write.buf.len());
            let len = rustix::io::write(&fd, write.buf)?;
            dbus.satisfy_write(len, &mut queue)?;
        }

        println!("read({})", read.buf.len());
        let len = rustix::io::read(&fd, read.buf)?;
        if let Some(message) = dbus.satisfy_read(len, &mut readbuf)? {
            if message.message_type == MessageType::Signal
                && message
                    .member
                    .is_some_and(|member| member == "NameAcquired")
            {
                println!("Name acquired!");
                let mut s = String::new();
                message.log(&mut s)?;
                println!("{s}");

                break;
            }
        }
    }

    println!("close()");
    drop(fd);

    Ok(())
}
