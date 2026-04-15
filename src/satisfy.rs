#[derive(Debug, PartialEq, Eq, Clone, Copy)]

pub enum Satisfy {
    Socket,
    Connect,
    Write,
    Read,
}
