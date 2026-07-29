use crate::{
    DBusError, EncodeError, MessageType, SliceMessageEncoder,
    messages::sni_client::dbusmenu::{DBusMenuData, DBusMenuItem, DBusMenuList},
};

pub struct GetLayout;

impl GetLayout {
    pub(crate) fn reply<'a>(
        buf: &'a mut [u8],
        serial: u32,
        destination: &str,
        data: &impl DBusMenuData,
        parent: i32,
        depth: i32,
    ) -> Result<&'a [u8], DBusError> {
        let menu_list = data.menu();

        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodReturn)?;
        encoder.set_reply_serial(serial)?;
        encoder.set_destination(destination)?;

        encoder.set_body_signature("u(ia{sv}av)")?;
        encoder.__dbus_begin_body()?;
        encoder.__dbus_write_u32(data.revision())?;
        if parent == 0 {
            traverse_root(&mut encoder, menu_list, depth)?;
        } else if let Some(item) = DBusMenuItem::find_in(menu_list, parent) {
            traverse_item(&mut encoder, item, depth)?;
        }
        let len = encoder.finish()?;
        buf.get(..len)
            .ok_or(DBusError::EncodeError(EncodeError::BufferTooSmall))
    }
}

fn traverse_root(
    encoder: &mut SliceMessageEncoder<'_>,
    list: &impl DBusMenuList,
    depth: i32,
) -> Result<(), EncodeError> {
    encoder.__dbus_align(8)?;
    encoder.__dbus_write_i32(0)?;

    let properties = encoder.__dbus_start_array(8)?;
    if list.iter().next().is_some() {
        encoder.__dbus_align(8)?;
        encoder.__dbus_write_string_like("children-display")?;
        encoder.__dbus_write_signature_value("s")?;
        encoder.__dbus_write_string_like("submenu")?;
    }
    encoder.__dbus_finish_array(properties)?;

    let children = encoder.__dbus_start_array(1)?;
    if let Some(depth) = child_depth(depth) {
        traverse_list(encoder, list, depth)?;
    }
    encoder.__dbus_finish_array(children)
}

fn traverse_list(
    encoder: &mut SliceMessageEncoder<'_>,
    list: &impl DBusMenuList,
    depth: i32,
) -> Result<(), EncodeError> {
    for item in list.iter() {
        traverse_item(encoder, item, depth)?;
    }
    Ok(())
}

fn traverse_item(
    encoder: &mut SliceMessageEncoder<'_>,
    item: &DBusMenuItem<'_, impl DBusMenuList>,
    depth: i32,
) -> Result<(), EncodeError> {
    encoder.__dbus_write_signature_value("(ia{sv}av)")?;
    encoder.__dbus_align(8)?;
    encoder.__dbus_write_i32(item.id())?;
    item.encode(encoder)?;

    let children = encoder.__dbus_start_array(1)?;
    if let Some(depth) = child_depth(depth)
        && let DBusMenuItem::Submenu { children, .. } = item
    {
        traverse_list(encoder, children, depth)?;
    }
    encoder.__dbus_finish_array(children)
}

const fn child_depth(depth: i32) -> Option<i32> {
    match depth {
        neg if neg < 0 => Some(-1),
        pos if pos > 0 => Some(pos.saturating_sub(1)),
        _ => None,
    }
}
