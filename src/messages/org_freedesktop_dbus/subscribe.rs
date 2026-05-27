use crate::{
    EncodeError,
    const_helpers::{get_range, try_},
    messages::org_freedesktop_dbus::{AddMatch, Rule},
};

/// A helper to build `AddMatch` message.
pub struct Subscribe;

impl Subscribe {
    /// Builds `AddMatch` message using provided options.
    ///
    /// # Errors
    ///
    /// Returns an error if given buffer is too short.
    pub const fn encode(
        buf: &mut [u8],
        sender: Option<&str>,
        path: Option<&str>,
        interface: Option<&str>,
        member: Option<&str>,
    ) -> Result<usize, EncodeError> {
        let mut rulebuf = [0; 1_024];
        let rulelen = try_!(Rule::fmt(&mut rulebuf, sender, interface, path, member));
        let Some(rule) = get_range(&rulebuf, 0, rulelen) else {
            return Err(EncodeError::BufferTooSmall);
        };
        let Ok(rule) = core::str::from_utf8(rule) else {
            unreachable!();
        };

        let len = try_!(AddMatch::encode(buf, rule));
        Ok(len)
    }
}
