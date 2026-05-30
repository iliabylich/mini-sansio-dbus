use crate::{
    EncodeError,
    const_helpers::{get_range, t_err},
    messages::org_freedesktop_dbus::{RemoveMatch, Rule},
};

/// A helper to build `RemoveMatch` message.
pub struct Unsubscribe;

impl Unsubscribe {
    /// Builds `RemoveMatch` message using provided options.
    ///
    /// # Errors
    ///
    /// Returns an error if given buffer is too short.
    pub const fn encode<'a>(
        buf: &'a mut [u8],
        sender: Option<&str>,
        path: Option<&str>,
        interface: Option<&str>,
        member: Option<&str>,
    ) -> Result<&'a [u8], EncodeError> {
        let mut rulebuf = [0; 1_024];
        let rulelen = t_err!(Rule::fmt(&mut rulebuf, sender, interface, path, member));
        let Some(rule) = get_range(&rulebuf, 0, rulelen) else {
            return Err(EncodeError::BufferTooSmall);
        };
        let Ok(rule) = core::str::from_utf8(rule) else {
            unreachable!();
        };

        RemoveMatch::encode(buf, rule)
    }
}
