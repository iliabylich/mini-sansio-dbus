use crate::{
    EncodeError, SliceMessageEncoder, dbus_body_fragment,
    messages::org_freedesktop_dbus::SetProperty, messaging::DBusEncode,
};

/// `RefreshRateMs` property of a device
pub struct RefreshRateMs;

impl DBusEncode for RefreshRateMs {
    type Args<'a> = (&'a str, u32);

    fn encode<'a>(
        (path, value): Self::Args<'_>,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EncodeError> {
        let write_value = |encoder: &mut SliceMessageEncoder<'_>| {
            dbus_body_fragment!(encoder, {
                variant<u32>(value),
            });
            Ok(())
        };
        SetProperty::encode(
            (
                "org.freedesktop.NetworkManager",
                path,
                "org.freedesktop.NetworkManager.Device.Statistics",
                "RefreshRateMs",
                &write_value,
            ),
            buf,
        )
    }
}
