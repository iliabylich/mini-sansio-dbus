use crate::{EncodeError, SliceMessageEncoder, messages::sni_client::dbusmenu::DBusMenuList};

/// A dbusmenu item
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DBusMenuItem<'a, List> {
    /// A simple clickable item
    Regular {
        /// Stable item id within this menu
        id: i32,
        /// User-visible label
        label: &'a str,
        /// Whether the item is enabled
        enabled: bool,
        /// Whether the item is visible
        visible: bool,
    },
    /// A checkbox item
    Checkbox {
        /// Stable item id within this menu
        id: i32,
        /// User-visible label
        label: &'a str,
        /// Whether the checkbox is checked
        checked: bool,
        /// Whether the item is enabled
        enabled: bool,
        /// Whether the item is visible
        visible: bool,
    },
    /// A radio item
    Radio {
        /// Stable item id within this menu
        id: i32,
        /// User-visible label
        label: &'a str,
        /// Whether the radio item is selected
        selected: bool,
        /// Whether the item is enabled
        enabled: bool,
        /// Whether the item is visible
        visible: bool,
    },
    /// A visual separator between groups of items
    Separator {
        /// Stable item id within this menu
        id: i32,
        /// Whether the item is visible
        visible: bool,
    },
    /// An item that opens a submenu
    Submenu {
        /// Stable item id within this menu
        id: i32,
        /// User-visible label
        label: &'a str,
        /// Whether the item is enabled
        enabled: bool,
        /// Whether the item is visible
        visible: bool,
        /// Child menu item list
        children: List,
    },
}

impl<'a, List> DBusMenuItem<'a, List> {
    pub(crate) const fn id(&self) -> i32 {
        match self {
            Self::Regular { id, .. }
            | Self::Checkbox { id, .. }
            | Self::Radio { id, .. }
            | Self::Separator { id, .. }
            | Self::Submenu { id, .. } => *id,
        }
    }

    pub(crate) fn encode(&self, encoder: &mut SliceMessageEncoder<'_>) -> Result<(), EncodeError> {
        let properties = encoder.__dbus_start_array(8)?;

        match self {
            DBusMenuItem::Regular {
                label,
                visible,
                enabled,
                ..
            } => {
                write_property_str(encoder, "label", label)?;
                write_property_bool(encoder, "visible", *visible)?;
                write_property_bool(encoder, "enabled", *enabled)?;
            }
            DBusMenuItem::Checkbox {
                label,
                checked,
                visible,
                enabled,
                ..
            } => {
                write_property_str(encoder, "label", label)?;
                write_property_str(encoder, "toggle-type", "checkmark")?;
                write_property_i32(encoder, "toggle-state", i32::from(*checked))?;
                write_property_bool(encoder, "visible", *visible)?;
                write_property_bool(encoder, "enabled", *enabled)?;
            }
            DBusMenuItem::Radio {
                label,
                selected,
                visible,
                enabled,
                ..
            } => {
                write_property_str(encoder, "label", label)?;
                write_property_str(encoder, "toggle-type", "radio")?;
                write_property_i32(encoder, "toggle-state", i32::from(*selected))?;
                write_property_bool(encoder, "visible", *visible)?;
                write_property_bool(encoder, "enabled", *enabled)?;
            }
            DBusMenuItem::Separator { visible, .. } => {
                write_property_str(encoder, "type", "separator")?;
                write_property_bool(encoder, "visible", *visible)?;
                write_property_bool(encoder, "enabled", false)?;
            }
            DBusMenuItem::Submenu {
                label,
                visible,
                enabled,
                ..
            } => {
                write_property_str(encoder, "label", label)?;
                write_property_str(encoder, "children-display", "submenu")?;
                write_property_bool(encoder, "visible", *visible)?;
                write_property_bool(encoder, "enabled", *enabled)?;
            }
        }

        encoder.__dbus_finish_array(properties)?;
        Ok(())
    }

    pub(crate) fn find_in(list: &'a List, id: i32) -> Option<&'a Self>
    where
        List: DBusMenuList,
    {
        for item in list.iter() {
            if item.id() == id {
                return Some(item);
            }
            if let DBusMenuItem::Submenu { children, .. } = item
                && let Some(item) = Self::find_in(children, id)
            {
                return Some(item);
            }
        }
        None
    }
}

fn write_property_str(
    encoder: &mut SliceMessageEncoder<'_>,
    name: &str,
    value: &str,
) -> Result<(), EncodeError> {
    encoder.__dbus_align(8)?;
    encoder.__dbus_write_string_like(name)?;
    encoder.__dbus_write_signature_value("s")?;
    encoder.__dbus_write_string_like(value)
}

fn write_property_bool(
    encoder: &mut SliceMessageEncoder<'_>,
    name: &str,
    value: bool,
) -> Result<(), EncodeError> {
    encoder.__dbus_align(8)?;
    encoder.__dbus_write_string_like(name)?;
    encoder.__dbus_write_signature_value("b")?;
    encoder.__dbus_write_bool(value)
}

fn write_property_i32(
    encoder: &mut SliceMessageEncoder<'_>,
    name: &str,
    value: i32,
) -> Result<(), EncodeError> {
    encoder.__dbus_align(8)?;
    encoder.__dbus_write_string_like(name)?;
    encoder.__dbus_write_signature_value("i")?;
    encoder.__dbus_write_i32(value)
}
