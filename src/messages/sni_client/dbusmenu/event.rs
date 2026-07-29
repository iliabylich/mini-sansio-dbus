/// A dbusmenu event kind sent by a host for a menu item
#[derive(Debug, Clone, Copy)]
pub enum DBusMenuEventKind<'a> {
    /// The item was activated
    Clicked,
    /// The item was hovered
    Hovered,
    /// A submenu item was opened
    Opened,
    /// A submenu item was closed
    Closed,
    /// An event kind not modeled by this library
    Other(&'a str),
}

impl<'a> DBusMenuEventKind<'a> {
    pub(crate) const fn from_str(event_id: &'a str) -> Self {
        match event_id.as_bytes() {
            b"clicked" => Self::Clicked,
            b"hovered" => Self::Hovered,
            b"opened" => Self::Opened,
            b"closed" => Self::Closed,
            _ => Self::Other(event_id),
        }
    }
}

/// An event sent by a dbusmenu host for a menu item
#[derive(Debug, Clone, Copy)]
pub struct DBusMenuEvent<'a> {
    /// Target menu item id
    pub id: i32,
    /// Event kind
    pub kind: DBusMenuEventKind<'a>,
    /// Host-provided event timestamp
    pub timestamp: u32,
}
