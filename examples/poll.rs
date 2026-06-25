use anyhow::{Result, ensure};
use mini_sansio_dbus::{
    Conf, DBusConnection, DBusConnector, DBusConnectorWants, DBusError, IncomingValue,
    OutgoingQueue, messages::org_freedesktop_dbus::Hello, messaging::property::Property, value_is,
};
use rustix::{
    event::{PollFd, PollFlags},
    net::{AddressFamily, SocketAddrUnix, SocketType},
};
use std::io::ErrorKind;

mod queue;
use queue::ExampleQueue;

fn main() -> Result<()> {
    let address = std::env::var("DBUS_SYSTEM_BUS_ADDRESS")
        .ok()
        .and_then(|address| address.split_once('=').map(|(_, path)| path.to_string()))
        .unwrap_or_else(|| String::from("/var/run/dbus/system_bus_socket"));
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

    let mut queue = ExampleQueue::new();
    queue.push_and_discard_reply::<Hello>(())?;
    let primary_connection_path_reply_handler = {
        let mut buf = [0; 1_024];
        let buf = PrimaryConnection.encode_get(&mut buf)?;
        queue.push_raw_and_prepare_for_reply(PrimaryConnection, buf)
    };
    let mut primary_connection_id_reply_handler = None;

    let mut dbus = DBusConnection::new(seq);
    println!("set_nonblocking()");
    rustix::io::ioctl_fionbio(&fd, true)?;

    loop {
        let (read, write) = dbus.wants(&mut queue, &mut readbuf)?;

        let mut poll_flags = PollFlags::empty();

        if let Some(write) = write {
            println!("write({})", write.buf.len());

            match rustix::io::write(&fd, write.buf) {
                Ok(len) => dbus.satisfy_write(len, &mut queue)?,
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    poll_flags |= PollFlags::OUT;
                }
                Err(err) => return Err(err.into()),
            }
        }

        println!("read({})", read.buf.len());
        match rustix::io::read(&fd, read.buf) {
            Ok(len) => {
                if let Some(message) = dbus.satisfy_read(len, &mut readbuf)? {
                    let mut s = String::new();
                    message.log(&mut s)?;
                    println!("{s}");

                    if let Some(primary_connection_path) =
                        primary_connection_path_reply_handler.handle(message)?
                    {
                        println!("Primary connection: {primary_connection_path}");

                        let mut buf = [0; 1_024];
                        let conn_id = ConnectionId {
                            conn_path: primary_connection_path,
                        };
                        let buf = conn_id.encode_get(&mut buf)?;
                        primary_connection_id_reply_handler =
                            Some(queue.push_raw_and_prepare_for_reply(conn_id, buf));
                    }

                    if let Some(primary_connection_id_reply_handler) =
                        primary_connection_id_reply_handler.as_ref()
                        && let Some(id) = primary_connection_id_reply_handler.handle(message)?
                    {
                        println!("Primary connection ID: {id}");
                        break;
                    }
                }
            }
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                poll_flags |= PollFlags::IN;
            }
            Err(err) => return Err(err.into()),
        }

        if !poll_flags.is_empty() {
            println!("poll({:?})", poll_flags);
            let mut pollfds = [PollFd::new(&fd, poll_flags)];
            let ready = rustix::event::poll(&mut pollfds, None)?;
            ensure!(ready > 0);
        }
    }
    println!("Connected to DBus!");

    Ok(())
}

#[derive(Clone)]
struct PrimaryConnection;
impl Property for PrimaryConnection {
    type Output<'a> = String;

    const DESTINATION: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager");
    const PATH: Conf<str, Self> = Conf::constant("/org/freedesktop/NetworkManager");
    const INTERFACE: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager");
    const PROPERTY_NAME: Conf<str, Self> = Conf::constant("PrimaryConnection");

    fn map(value: IncomingValue<'_>) -> Result<Self::Output<'_>, DBusError> {
        value_is!(value, IncomingValue::ObjectPath(value));
        Ok(value.to_string())
    }
}

#[derive(Clone)]
struct ConnectionId {
    conn_path: String,
}
impl Property for ConnectionId {
    type Output<'a> = String;

    const DESTINATION: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager");
    const PATH: Conf<str, Self> = Conf::dynamic(|this| this.conn_path.as_str());
    const INTERFACE: Conf<str, Self> =
        Conf::constant("org.freedesktop.NetworkManager.Connection.Active");
    const PROPERTY_NAME: Conf<str, Self> = Conf::constant("Id");

    fn map(value: IncomingValue<'_>) -> Result<Self::Output<'_>, DBusError> {
        value_is!(value, IncomingValue::String(value));
        Ok(value.to_string())
    }
}
