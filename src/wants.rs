/// Represents an operation that `DBusConnector` wants YOU to perform.
///
/// Usually the flow should be:
///
/// ```ignore
/// let mut readbuf = [0; 1_024];
/// match connector.wants(&mut readbuf)? {
///     DBusConnectorWants::Read { buf, .. } => {
///         let len = read(fd, buf)?;
///         connector.satisfy_read(len, readbuf)?;
///     }
///     DBusConnectorWants::Write { buf, .. } => {
///         let len = write(fd, buf);
///         if let Some(seq) = connector.satisfy_write(len)? {
///             // Connected
///         }
///     }
/// }
/// ```
#[derive(Debug)]
pub enum DBusConnectorWants<'r, 'w> {
    /// A `read()` opertion
    Read {
        /// `buf` argument of the `read()` call
        buf: &'r mut [u8],
        /// sequence number of a request
        seq: u64,
    },
    /// A `write()` opertion
    Write {
        /// `buf` argument of the `write()` call
        buf: &'w [u8],
        /// sequence number of a request
        ///
        seq: u64,
    },
}

/// Represents a `read()` operation that `DBusConnection` wants YOU to perform.
///
/// ```ignore
/// let (DBusWantsRead { buf, .. }, _dbus_wants_write) = conn.wants()?;
/// let len = read(fd, buf)?;
/// let maybe_message = connector.satisfy_read(len, readbuf)?;
/// ```
#[derive(Debug)]
pub struct DBusWantsRead<'r> {
    /// `buf` argument of the `read()` call
    pub buf: &'r mut [u8],
    /// sequence number of a request
    pub seq: u64,
}

/// Represents a `write()` operation that `DBusConnection` wants YOU to perform.
///
/// ```ignore
/// let (_dbus_wants_read, DBusWantsWrite { buf, .. }, _) = conn.wants()?;
/// let len = write(fd, buf)?;
/// connector.satisfy_write(len, &mut queue)?;
/// ```
#[derive(Debug)]
pub struct DBusWantsWrite<'w> {
    /// `buf` argument of the `write()` call
    pub buf: &'w [u8],
    /// sequence number of a request
    ///
    pub seq: u64,
}
