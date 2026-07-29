use anyhow::{Context, Result};
use mini_sansio_dbus::{
    DBusConnection, DBusConnector, DBusConnectorWants, DBusError, IncomingMessage, MessageType,
    OutgoingQueue,
    messages::{
        EmptyMethodReturn,
        org_freedesktop_dbus::{Hello, NameHasOwner, NameOwnerChangedSubscribe, RequestName},
        sni_client::{
            dbusmenu::{
                DBusMenuData, DBusMenuEvent, DBusMenuEventKind, DBusMenuItem, DBusMenuList,
                LayoutUpdatedSignal, StatusNotifierMenuHandler,
            },
            sni::{
                IconPixmap, NewIconSignal, RegisterStatusNotifierItem, StatusNotifierActivateEvent,
                StatusNotifierItemCategory, StatusNotifierItemData, StatusNotifierItemHandler,
                StatusNotifierItemStatus, StatusNotifierWatcher,
            },
        },
    },
    messaging::reply_handler::ReplyHandler,
};
use rustix::{
    event::{PollFd, PollFlags},
    net::{AddressFamily, SocketAddrUnix, SocketType},
};
use std::{io::ErrorKind, os::fd::OwnedFd};

mod queue;
use queue::ExampleQueue;

mod timer;
use timer::Timer;

const DBUS_NAME: &str = "org.freedesktop.StatusNotifierItem-demo";
const KSNI: &str = "org.kde.StatusNotifierWatcher";

fn main() -> Result<()> {
    let fd = connect_to_session_dbus()?;
    let mut readbuf = [0; 8 * 1_024];
    let seq = authenticate(&fd, &mut readbuf)?;
    rustix::io::ioctl_fionbio(&fd, true)?;
    let timer = Timer::new()?;

    let mut dbus = DBusConnection::new(seq);
    let mut queue = ExampleQueue::new();

    queue.push_without_reply::<Hello>(())?;
    queue.push_without_reply::<RequestName>(DBUS_NAME)?;
    queue.push_without_reply::<NameOwnerChangedSubscribe>(())?;
    let sni_has_owner_reply = queue.push_with_reply(NameHasOwner, KSNI)?;

    let mut registration = Registration::Unset;
    let sni = StatusNotifierItemHandler::new(DBUS_NAME);
    let mut tray_menu = ExampleTrayMenu::new();

    while !tray_menu.exit_requested() {
        let (message, timer_is_ready) = {
            let (read, write) = dbus.wants(&queue, &mut readbuf)?;

            let mut pollfds = [
                PollFd::new(
                    &fd,
                    if write.is_some() {
                        PollFlags::IN | PollFlags::OUT
                    } else {
                        PollFlags::IN
                    },
                ),
                PollFd::new(&timer, PollFlags::IN),
            ];
            rustix::event::poll(&mut pollfds, None)?;
            check_poll_errors("DBus socket", pollfds[0].revents())?;
            check_poll_errors("timerfd", pollfds[1].revents())?;

            let dbus_is_readable = pollfds[0].revents().contains(PollFlags::IN);
            let dbus_is_writable = pollfds[0].revents().contains(PollFlags::OUT);
            let timer_is_ready = pollfds[1].revents().contains(PollFlags::IN);

            if dbus_is_writable && let Some(write) = write {
                match rustix::io::write(&fd, write.buf) {
                    Ok(len) => dbus.satisfy_write(len, &mut queue)?,
                    Err(err) if err.kind() == ErrorKind::WouldBlock => {}
                    Err(err) => return Err(err.into()),
                }
            }

            let message = if dbus_is_readable {
                match rustix::io::read(&fd, read.buf) {
                    Ok(0) => anyhow::bail!("DBus connection closed"),
                    Ok(len) => dbus.satisfy_read(len, &readbuf)?,
                    Err(err) if err.kind() == ErrorKind::WouldBlock => None,
                    Err(err) => return Err(err.into()),
                }
            } else {
                None
            };

            (message, timer_is_ready)
        };

        if let Some(message) = message {
            if let Some(has_owner) = sni_has_owner_reply.handle(message)? {
                if has_owner {
                    registration.host_appeared(&mut queue)?;
                } else {
                    registration.host_disappeared();
                }
            }

            if let Some(event) = StatusNotifierWatcher::handle(message)? {
                match event {
                    StatusNotifierWatcher::Appeared { .. } => {
                        registration.host_appeared(&mut queue)?;
                    }
                    StatusNotifierWatcher::Disappeared { .. } => {
                        registration.host_disappeared();
                    }
                }
            }

            registration.handle_message(message)?;

            let mut reply = [0; 8 * 1_024];
            if let Some(reply) = StatusNotifierMenuHandler::handle(
                &mut reply,
                message,
                DBUS_NAME,
                "/StatusNotifierItem/Menu",
                &mut tray_menu,
            )? {
                let _ = queue.push_raw(reply);
            } else if let Some(reply) = sni.handle(&mut reply, message, &tray_menu)? {
                let _ = queue.push_raw(reply);
            } else if StatusNotifierActivateEvent::handle(message) {
                println!("Activate called");
                let sender = message.sender.ok_or(DBusError::NoSender)?;
                queue.push_without_reply::<EmptyMethodReturn>((sender, message.serial))?;
            }
        }

        if timer_is_ready {
            println!("Tick");
            timer.read()?;
            tray_menu.flip_state();
        }

        if tray_menu.has_pending_layout_update() {
            queue.push_without_reply::<LayoutUpdatedSignal>((
                "/StatusNotifierItem/Menu",
                tray_menu.revision(),
                0,
            ))?;
        }

        if tray_menu.has_pending_icon_update() {
            queue.push_without_reply::<NewIconSignal>(())?;
        }
    }

    Ok(())
}

fn check_poll_errors(name: &str, events: PollFlags) -> Result<()> {
    if events.contains(PollFlags::NVAL) {
        anyhow::bail!("{name} poll failed: invalid fd");
    }
    if events.contains(PollFlags::ERR) {
        anyhow::bail!("{name} poll failed: fd error");
    }
    if events.intersects(PollFlags::HUP | PollFlags::RDHUP) {
        anyhow::bail!("{name} poll failed: hangup");
    }
    Ok(())
}

fn connect_to_session_dbus() -> Result<OwnedFd> {
    let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")?;
    let (_, address) = address
        .split_once('=')
        .context("malformed $DBUS_SESSION_BUS_ADDRESS")?;
    let address = SocketAddrUnix::new(address)?;
    let fd = rustix::net::socket(AddressFamily::UNIX, SocketType::STREAM, None)?;
    rustix::net::connect(&fd, &address)?;
    Ok(fd)
}

fn authenticate(fd: &OwnedFd, readbuf: &mut [u8]) -> Result<u64> {
    let mut connector = DBusConnector::new();
    loop {
        let wants = connector.wants(readbuf)?;
        match wants {
            DBusConnectorWants::Read { buf, .. } => {
                let len = rustix::io::read(fd, buf)?;
                connector.satisfy_read(len, readbuf)?;
            }
            DBusConnectorWants::Write { buf, .. } => {
                let len = rustix::io::write(fd, buf)?;
                if let Some(seq) = connector.satisfy_write(len)? {
                    return Ok(seq);
                }
            }
        }
    }
}

enum Registration {
    Unset,
    Registering(ReplyHandler<RegisterStatusNotifierItem>),
    Registered,
}

impl Registration {
    fn host_appeared(&mut self, queue: &mut ExampleQueue) -> Result<()> {
        let Self::Unset = self else {
            anyhow::bail!("invalid registration transition: host appeared while not unset");
        };
        let handler = queue.push_with_reply(RegisterStatusNotifierItem, DBUS_NAME)?;
        *self = Self::Registering(handler);
        println!("registering {DBUS_NAME}");
        Ok(())
    }

    fn host_disappeared(&mut self) {
        *self = Self::Unset;
    }

    fn handle_message(&mut self, message: IncomingMessage<'_>) -> Result<()> {
        let Self::Registering(handler) = self else {
            return Ok(());
        };

        if matches!(message.message_type, MessageType::MethodReturn)
            && message
                .reply_serial
                .is_some_and(|reply_serial| reply_serial == handler.serial)
        {
            *self = Self::Registered;
            println!("registered");
        };
        Ok(())
    }
}

struct ExampleTrayMenu {
    flip: bool,
    revision: u32,
    layout_update_pending: bool,
    icon_update_pending: bool,
    checkbox_checked: bool,
    radio_selected: i32,
    exit_requested: bool,
    menu: MenuList<'static>,
}

impl ExampleTrayMenu {
    fn new() -> Self {
        let flip = false;
        let checkbox_checked = false;
        let radio_selected = ID_STATEFUL_RADIO_B;
        Self {
            flip,
            revision: 1,
            layout_update_pending: false,
            icon_update_pending: false,
            checkbox_checked,
            radio_selected,
            exit_requested: false,
            menu: MenuList::new(flip, checkbox_checked, radio_selected),
        }
    }

    fn flip_state(&mut self) {
        self.flip = !self.flip;
        self.rebuild_menu();
        self.icon_update_pending = true;
    }

    const fn exit_requested(&self) -> bool {
        self.exit_requested
    }

    fn has_pending_layout_update(&mut self) -> bool {
        core::mem::take(&mut self.layout_update_pending)
    }

    fn has_pending_icon_update(&mut self) -> bool {
        core::mem::take(&mut self.icon_update_pending)
    }

    fn rebuild_menu(&mut self) {
        self.revision = self.revision.saturating_add(1);
        self.layout_update_pending = true;
        self.menu = MenuList::new(self.flip, self.checkbox_checked, self.radio_selected);
    }
}

impl StatusNotifierItemData for ExampleTrayMenu {
    fn id(&self) -> &str {
        "mini-sansio-dbus-example"
    }

    fn title(&self) -> &str {
        "mini-sansio-dbus"
    }

    fn status(&self) -> StatusNotifierItemStatus {
        StatusNotifierItemStatus::Active
    }

    fn icon_name(&self) -> &str {
        if self.flip {
            "network-wireless-signal-excellent"
        } else {
            "network-wireless-signal-none"
        }
    }

    fn icon_pixmap(&self) -> Option<IconPixmap<'_>> {
        let argb = if self.flip { &GREEN_ICON } else { &BLUE_ICON };
        Some(IconPixmap {
            width: 2,
            height: 2,
            argb,
        })
    }

    fn menu(&self) -> &str {
        "/StatusNotifierItem/Menu"
    }

    fn category(&self) -> StatusNotifierItemCategory {
        StatusNotifierItemCategory::ApplicationStatus
    }

    fn item_is_menu(&self) -> bool {
        false
    }
}

impl DBusMenuData for ExampleTrayMenu {
    type List = MenuList<'static>;

    fn revision(&self) -> u32 {
        self.revision
    }

    fn menu(&self) -> &Self::List {
        &self.menu
    }

    fn event(&mut self, event: DBusMenuEvent<'_>) {
        println!(
            "menu event: id={}, event={:?}, timestamp={}",
            event.id, event.kind, event.timestamp
        );
        if !matches!(event.kind, DBusMenuEventKind::Clicked) {
            return;
        }
        match event.id {
            ID_CLICKABLE => {
                println!("clickable item clicked");
            }
            ID_STATEFUL_CHECKBOX => {
                self.checkbox_checked = !self.checkbox_checked;
                self.rebuild_menu();
            }
            ID_STATEFUL_RADIO_A | ID_STATEFUL_RADIO_B => {
                self.radio_selected = event.id;
                self.rebuild_menu();
            }
            ID_EXIT => {
                println!("exit requested");
                self.exit_requested = true;
            }
            _ => {}
        }
    }
}

struct MenuList<'a>(Vec<DBusMenuItem<'a, MenuList<'a>>>);

impl MenuList<'static> {
    fn new(flip: bool, checkbox_checked: bool, radio_selected: i32) -> Self {
        Self(vec![
            DBusMenuItem::Regular {
                id: ID_DYNAMIC_TEXT,
                label: if flip {
                    "dynamic text 2"
                } else {
                    "dynamic text 1"
                },
                enabled: true,
                visible: true,
            },
            DBusMenuItem::Checkbox {
                id: ID_DYNAMIC_CHECKBOX,
                label: "dynamic checkbox",
                checked: flip,
                enabled: true,
                visible: true,
            },
            DBusMenuItem::Radio {
                id: ID_DYNAMIC_RADIO,
                label: "dynamic radio",
                selected: flip,
                enabled: true,
                visible: true,
            },
            DBusMenuItem::Separator {
                id: ID_SEPARATOR,
                visible: true,
            },
            DBusMenuItem::Regular {
                id: ID_CLICKABLE,
                label: "click me",
                enabled: true,
                visible: true,
            },
            DBusMenuItem::Submenu {
                id: ID_SUBMENU,
                label: "submenu",
                enabled: true,
                visible: true,
                children: MenuList(vec![
                    DBusMenuItem::Regular {
                        id: ID_NESTED_1,
                        label: "nested 1",
                        enabled: true,
                        visible: true,
                    },
                    DBusMenuItem::Regular {
                        id: ID_NESTED_2,
                        label: "nested 2",
                        enabled: true,
                        visible: true,
                    },
                ]),
            },
            DBusMenuItem::Checkbox {
                id: ID_STATEFUL_CHECKBOX,
                label: "stateful checkbox",
                checked: checkbox_checked,
                enabled: true,
                visible: true,
            },
            DBusMenuItem::Radio {
                id: ID_STATEFUL_RADIO_A,
                label: "stateful radio 1",
                selected: radio_selected == ID_STATEFUL_RADIO_A,
                enabled: true,
                visible: true,
            },
            DBusMenuItem::Radio {
                id: ID_STATEFUL_RADIO_B,
                label: "stateful radio 2",
                selected: radio_selected == ID_STATEFUL_RADIO_B,
                enabled: true,
                visible: true,
            },
            DBusMenuItem::Regular {
                id: ID_EXIT,
                label: "exit",
                enabled: true,
                visible: true,
            },
        ])
    }
}

impl DBusMenuList for MenuList<'_> {
    fn iter(&self) -> impl Iterator<Item = &DBusMenuItem<'_, Self>> {
        self.0.iter()
    }
}

const ID_DYNAMIC_TEXT: i32 = 1;
const ID_DYNAMIC_CHECKBOX: i32 = 2;
const ID_DYNAMIC_RADIO: i32 = 3;
const ID_SEPARATOR: i32 = 4;
const ID_CLICKABLE: i32 = 5;
const ID_SUBMENU: i32 = 6;
const ID_NESTED_1: i32 = 7;
const ID_NESTED_2: i32 = 8;
const ID_STATEFUL_CHECKBOX: i32 = 9;
const ID_STATEFUL_RADIO_A: i32 = 10;
const ID_STATEFUL_RADIO_B: i32 = 11;
const ID_EXIT: i32 = 12;

const BLUE_ICON: [u8; 16] = [
    0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00,
];
const GREEN_ICON: [u8; 16] = [
    0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00,
];
