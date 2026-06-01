use crate::{EncodeError, dbus_body_fragment, messages::org_freedesktop_dbus::SetProperty};

/// `RefreshRateMs` property of a device
pub struct RefreshRateMs;

impl RefreshRateMs {
    /// Encodes `SetProperty` call for `RefreshRateMs` property of a given device to `value`
    ///
    /// # Errors
    ///
    /// Returns an error if encoded message doesn't fit into a buffer
    pub fn encode_set_property<'a>(
        buf: &'a mut [u8],
        path: &str,
        value: u32,
    ) -> Result<&'a [u8], EncodeError> {
        SetProperty::encode(
            buf,
            "org.freedesktop.NetworkManager",
            path,
            "org.freedesktop.NetworkManager.Device.Statistics",
            "RefreshRateMs",
            |encoder| {
                dbus_body_fragment!(encoder, {
                    variant<u32>(value),
                });
                Ok(())
            },
        )
    }
}
