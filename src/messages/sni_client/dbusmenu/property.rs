use crate::{EncodeError, SliceMessageEncoder};

#[derive(Clone, Copy, Debug)]
pub enum Property {
    Version,
    TextDirection,
    Status,
    IconThemePath,
}

impl Property {
    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s {
            "Version" => Some(Self::Version),
            "TextDirection" => Some(Self::TextDirection),
            "Status" => Some(Self::Status),
            "IconThemePath" => Some(Self::IconThemePath),
            _ => None,
        }
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Version => "Version",
            Self::TextDirection => "TextDirection",
            Self::Status => "Status",
            Self::IconThemePath => "IconThemePath",
        }
    }

    pub(crate) fn encode_key_value(
        self,
        encoder: &mut SliceMessageEncoder<'_>,
    ) -> Result<(), EncodeError> {
        encoder.__dbus_align(8)?;
        self.encode_key(encoder)?;
        self.encode_value(encoder)?;
        Ok(())
    }

    pub(crate) fn encode_key(
        self,
        encoder: &mut SliceMessageEncoder<'_>,
    ) -> Result<(), EncodeError> {
        encoder.__dbus_write_string_like(self.as_str())?;
        Ok(())
    }

    pub(crate) fn encode_value(
        self,
        encoder: &mut SliceMessageEncoder<'_>,
    ) -> Result<(), EncodeError> {
        match self {
            Self::Version => {
                encoder.__dbus_write_signature_value("u")?;
                encoder.__dbus_write_u32(4)?;
            }
            Self::TextDirection => {
                encoder.__dbus_write_signature_value("s")?;
                encoder.__dbus_write_string_like("ltr")?;
            }
            Self::Status => {
                encoder.__dbus_write_signature_value("s")?;
                encoder.__dbus_write_string_like("normal")?;
            }
            Self::IconThemePath => {
                encoder.__dbus_write_signature_value("as")?;
                let paths = encoder.__dbus_start_array(4)?;
                encoder.__dbus_finish_array(paths)?;
            }
        }
        Ok(())
    }
}
