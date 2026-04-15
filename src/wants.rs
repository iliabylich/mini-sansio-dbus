#[derive(Debug)]
pub enum Wants {
    Socket {
        domain: i32,
        r#type: i32,
    },
    Connect {
        fd: i32,
        addr: *const libc::sockaddr,
        addrlen: u32,
    },
    Read {
        fd: i32,
        buf: *mut u8,
        len: usize,
    },
    Write {
        fd: i32,
        buf: *const u8,
        len: usize,
    },
    ReadWrite {
        fd: i32,
        readbuf: *mut u8,
        readlen: usize,
        writebuf: *const u8,
        writelen: usize,
    },
}
