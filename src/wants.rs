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
pub enum DBusWants {
    /// A `socket()` opertion
    Socket {
        /// `domain` argument of the `socket()` call
        domain: i32,
        /// `type` argument of the `socket()` call
        r#type: i32,
        /// sequence number of a request
        seq: u64,
    },
    /// A `connect()` opertion
    Connect {
        /// `fd` argument of the `connect()` call
        fd: i32,
        /// `addr` argument of the `connect()` call
        addr: *const libc::sockaddr,
        /// `addrlenm` argument of the `connect()` call
        addrlen: u32,
        /// sequence number of a request
        seq: u64,
    },
    /// A `read()` opertion
    Read {
        /// `fd` argument of the `read()` call
        fd: i32,
        /// `buf` argument of the `read()` call
        buf: *mut u8,
        /// `len` argument of the `read()` call
        len: usize,
        /// sequence number of a request
        seq: u64,
    },
    /// A `write()` opertion
    Write {
        /// `fd` argument of the `write()` call
        fd: i32,
        /// `buf` argument of the `write()` call
        buf: *const u8,
        /// `len` argument of the `write()` call
        len: usize,
        /// sequence number of a request
        seq: u64,
    },
    /// A combination of `read()` +  `write()` opertions
    ReadWrite {
        /// `fd` argument of `read()` / `write()` call
        fd: i32,
        /// `buf` argument of the `read()` call
        readbuf: *mut u8,
        /// `len` argument of the `read()` call
        readlen: usize,
        /// sequence number of the `read()` call
        readseq: u64,
        /// `buf` argument of the `write()` call
        writebuf: *const u8,
        /// `len` argument of the `write()` call
        writelen: usize,
        /// sequence number of the `write()` call
        writeseq: u64,
    },
}
