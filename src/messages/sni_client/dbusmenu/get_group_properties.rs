use crate::{
    DBusError, EncodeError, IncomingArrayValue, IncomingValue, MessageType, SliceMessageEncoder,
    messages::sni_client::dbusmenu::{DBusMenuData, DBusMenuItem, DBusMenuList},
    value_is,
};

pub struct GetGroupProperties;

impl GetGroupProperties {
    pub(crate) fn reply<'a, S>(
        buf: &'a mut [u8],
        serial: u32,
        destination: &str,
        data: &impl DBusMenuData<S>,
        ids: IncomingArrayValue<'_>,
    ) -> Result<&'a [u8], DBusError>
    where
        S: AsRef<str> + 'static,
    {
        let menu_list = data.menu();

        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodReturn)?;
        encoder.set_reply_serial(serial)?;
        encoder.set_destination(destination)?;
        encoder.set_body_signature("a(ia{sv})")?;
        encoder.__dbus_begin_body()?;
        let items = encoder.__dbus_start_array(8)?;

        if ids.is_empty() {
            traverse_root_properties(&mut encoder, menu_list)?;
        } else {
            let mut ids = ids.items_iter();
            while let Some(value) = ids.try_next()? {
                value_is!(value, IncomingValue::Int32(id));
                if id == 0 {
                    traverse_root_properties(&mut encoder, menu_list)?;
                } else if let Some(item) = DBusMenuItem::find_in(menu_list, id) {
                    traverse_item_properties(&mut encoder, item)?;
                }
            }
        }

        encoder.__dbus_finish_array(items)?;

        let len = encoder.finish()?;
        buf.get(..len)
            .ok_or(DBusError::EncodeError(EncodeError::BufferTooSmall))
    }
}

fn traverse_root_properties<S>(
    encoder: &mut SliceMessageEncoder<'_>,
    list: &impl DBusMenuList<S>,
) -> Result<(), EncodeError>
where
    S: AsRef<str> + 'static,
{
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
    traverse_list_properties(encoder, list)
}

fn traverse_list_properties<S>(
    encoder: &mut SliceMessageEncoder<'_>,
    list: &impl DBusMenuList<S>,
) -> Result<(), EncodeError>
where
    S: AsRef<str> + 'static,
{
    for item in list.iter() {
        traverse_item_properties(encoder, item)?;
    }
    Ok(())
}

fn traverse_item_properties<S>(
    encoder: &mut SliceMessageEncoder<'_>,
    item: &DBusMenuItem<impl DBusMenuList<S>, S>,
) -> Result<(), EncodeError>
where
    S: AsRef<str> + 'static,
{
    encoder.__dbus_align(8)?;
    encoder.__dbus_write_i32(item.id())?;
    item.encode(encoder)?;
    if let DBusMenuItem::Submenu { children, .. } = item {
        traverse_list_properties(encoder, children)?;
    }
    Ok(())
}
