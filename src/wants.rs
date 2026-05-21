use rustix::net::{AddressFamily, SocketAddrUnix, SocketType};

/// Represents an operation that `DBus` connection wants YOU to perform.
///
/// Usually the flow should be:
///
/// ```ignore
/// if let Some(wants) = conn.wants()) {
///     // do a syscall
/// }
/// // ... later once the operation is done
/// conn.satisfy(Satisfy::<OneThatMatchesWants>);
/// ```
#[derive(Debug)]
pub enum DBusWants<'readbuf, 'writebuf> {
    /// A `socket()` opertion
    Socket {
        /// `domain` argument of the `socket()` call
        domain: AddressFamily,
        /// `type` argument of the `socket()` call
        r#type: SocketType,
        /// sequence number of a request
        seq: u64,
    },
    /// A `connect()` opertion
    Connect {
        /// `addr` argument of the `connect()` call
        addr: SocketAddrUnix,
        /// sequence number of a request
        seq: u64,
    },
    /// A `read()` opertion
    Read {
        /// `buf` argument of the `read()` call
        buf: &'readbuf mut [u8],
        /// sequence number of a request
        seq: u64,
    },
    /// A `write()` opertion
    Write {
        /// `buf` argument of the `write()` call
        buf: &'writebuf [u8],
        /// sequence number of a request
        ///
        seq: u64,
    },
    /// A combination of `read()` +  `write()` opertions
    ReadWrite {
        /// `buf` argument of the `read()` call
        readbuf: &'readbuf mut [u8],
        /// sequence number of the `read()` call
        readseq: u64,
        /// `buf` argument of the `write()` call
        writebuf: &'writebuf [u8],
        /// sequence number of the `write()` call
        writeseq: u64,
    },
}
