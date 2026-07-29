use crate::{
    EncodeError, SliceMessageEncoder,
    messages::sni_client::sni::{IconPixmap, StatusNotifierItemData},
};

#[derive(Clone, Copy)]
pub enum Property {
    Category,
    Id,
    Title,
    Status,
    IconName,
    IconPixmap,
    Menu,
    ItemIsMenu,
}

impl Property {
    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s {
            "Category" => Some(Self::Category),
            "Id" => Some(Self::Id),
            "Title" => Some(Self::Title),
            "Status" => Some(Self::Status),
            "IconName" => Some(Self::IconName),
            "IconPixmap" => Some(Self::IconPixmap),
            "Menu" => Some(Self::Menu),
            "ItemIsMenu" => Some(Self::ItemIsMenu),
            _ => None,
        }
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Category => "Category",
            Self::Id => "Id",
            Self::Title => "Title",
            Self::Status => "Status",
            Self::IconName => "IconName",
            Self::IconPixmap => "IconPixmap",
            Self::Menu => "Menu",
            Self::ItemIsMenu => "ItemIsMenu",
        }
    }

    pub(crate) fn encode_key_value(
        self,
        encoder: &mut SliceMessageEncoder<'_>,
        data: &impl StatusNotifierItemData,
    ) -> Result<(), EncodeError> {
        encoder.__dbus_align(8)?;
        self.encode_key(encoder)?;
        self.encode_value(encoder, data)?;
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
        data: &impl StatusNotifierItemData,
    ) -> Result<(), EncodeError> {
        match self {
            Self::Category => {
                encoder.__dbus_write_signature_value("s")?;
                encoder.__dbus_write_string_like(data.category().as_str())
            }
            Self::Id => {
                encoder.__dbus_write_signature_value("s")?;
                encoder.__dbus_write_string_like(data.id())
            }
            Self::Title => {
                encoder.__dbus_write_signature_value("s")?;
                encoder.__dbus_write_string_like(data.title())
            }
            Self::Status => {
                encoder.__dbus_write_signature_value("s")?;
                encoder.__dbus_write_string_like(data.status().as_str())
            }
            Self::IconName => {
                encoder.__dbus_write_signature_value("s")?;
                encoder.__dbus_write_string_like(data.icon_name())
            }
            Self::IconPixmap => {
                encoder.__dbus_write_signature_value("a(iiay)")?;
                IconPixmap::encode(encoder, data.icon_pixmap())
            }
            Self::Menu => {
                encoder.__dbus_write_signature_value("o")?;
                encoder.__dbus_write_string_like(data.menu())
            }
            Self::ItemIsMenu => {
                encoder.__dbus_write_signature_value("b")?;
                encoder.__dbus_write_bool(data.item_is_menu())
            }
        }
    }
}
