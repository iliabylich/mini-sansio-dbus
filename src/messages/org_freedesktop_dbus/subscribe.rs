use crate::{
    EncodeError,
    messages::org_freedesktop_dbus::{AddMatch, Rule},
    messaging::DBusEncode,
};

/// A helper to build `AddMatch` message.
pub struct Subscribe;

impl DBusEncode for Subscribe {
    type Args<'a> = (
        Option<&'a str>,
        Option<&'a str>,
        Option<&'a str>,
        Option<&'a str>,
    );

    fn encode<'a>(
        (sender, path, interface, member): Self::Args<'_>,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EncodeError> {
        let mut rulebuf = [0; 1_024];
        let rulelen = Rule::fmt(&mut rulebuf, sender, interface, path, member)?;
        let rule = rulebuf.get(0..rulelen).ok_or(EncodeError::BufferTooSmall)?;
        let Ok(rule) = core::str::from_utf8(rule) else {
            unreachable!();
        };

        AddMatch::encode(rule, buf)
    }
}
