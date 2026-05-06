/// Represents completion of a previously request operation (`Wants`)
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]

pub enum Satisfy {
    /// A `socket()` operation
    Socket,
    /// A `connect()` operation
    Connect,
    /// A `write()` operation
    Write,
    /// A `read()` operation
    Read,
}
