use crate::{
    EncodeError,
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
    pub fn encode<'a>(
        buf: &'a mut [u8],
        sender: Option<&str>,
        path: Option<&str>,
        interface: Option<&str>,
        member: Option<&str>,
    ) -> Result<&'a [u8], EncodeError> {
        let mut rulebuf = [0; 1_024];
        let rulelen = Rule::fmt(&mut rulebuf, sender, interface, path, member)?;
        let rule = rulebuf.get(0..rulelen).ok_or(EncodeError::BufferTooSmall)?;
        let Ok(rule) = core::str::from_utf8(rule) else {
            unreachable!();
        };

        AddMatch::encode(buf, rule)
    }
}
