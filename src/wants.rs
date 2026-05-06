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
pub enum Wants {
    /// A `socket()` opertion
    Socket {
        /// `domain` argument of the `socket()` call
        domain: i32,
        /// `type` argument of the `socket()` call
        r#type: i32,
    },
    /// A `connect()` opertion
    Connect {
        /// `fd` argument of the `connect()` call
        fd: i32,
        /// `addr` argument of the `connect()` call
        addr: *const libc::sockaddr,
        /// `addrlenm` argument of the `connect()` call
        addrlen: u32,
    },
    /// A `read()` opertion
    Read {
        /// `fd` argument of the `read()` call
        fd: i32,
        /// `buf` argument of the `read()` call
        buf: *mut u8,
        /// `len` argument of the `read()` call
        len: usize,
    },
    /// A `write()` opertion
    Write {
        /// `fd` argument of the `write()` call
        fd: i32,
        /// `buf` argument of the `write()` call
        buf: *const u8,
        /// `len` argument of the `write()` call
        len: usize,
    },
    /// A combination of `read()` +  `write()` opertions
    ReadWrite {
        /// `fd` argument of `read()` / `write()` calls
        fd: i32,
        /// `buf` argument of `read()` calls
        readbuf: *mut u8,
        /// `len` argument of `read()` calls
        readlen: usize,
        /// `buf` argument of `write()` calls
        writebuf: *const u8,
        /// `len` argument of `write()` calls
        writelen: usize,
    },
}
