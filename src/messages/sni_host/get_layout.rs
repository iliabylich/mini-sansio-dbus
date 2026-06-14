use crate::{
    DBusError, EncodeError, IncomingArrayValue, IncomingBody, IncomingValue, MessageType,
    SliceMessageEncoder, dbus_body,
    messaging::{DBusEncode, reply_handler::HandleReply},
    value_is,
};
use core::marker::PhantomData;

/// A helper struct that send and handle reply of `GetLayout` method call
#[derive(Clone)]
pub struct GetLayout<L, I, D>
where
    L: GetLayoutList<Item = I>,
    I: GetLayoutItem<List = L>,
    D: AsRef<str> + Clone,
{
    destination: D,
    _marker: PhantomData<(L, I)>,
}

impl<L, I, D> DBusEncode for GetLayout<L, I, D>
where
    L: GetLayoutList<Item = I>,
    I: GetLayoutItem<List = L>,
    D: AsRef<str> + Clone,
{
    type Args<'a> = (&'a str, &'a str);

    fn encode<'a>(
        (destination, path): Self::Args<'_>,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall)?;
        encoder.set_destination(destination)?;
        encoder.set_path(path)?;
        encoder.set_interface("com.canonical.dbusmenu")?;
        encoder.set_member("GetLayout")?;
        dbus_body!(encoder, {
            i32(0),
            i32(-1),
            array<str> [
                "type",
                "label",
                "enabled",
                "visible",
                "toggle-type",
                "toggle-state",
                "children-display",
            ],
        });
        let len = encoder.finish()?;
        buf.get(0..len).ok_or(EncodeError::BufferTooSmall)
    }
}

impl<L, I, D> GetLayout<L, I, D>
where
    L: GetLayoutList<Item = I>,
    I: GetLayoutItem<List = L>,
    D: AsRef<str> + Clone,
{
    /// Constructor
    pub const fn new(destination: D) -> Self {
        Self {
            destination,
            _marker: PhantomData,
        }
    }
}

const ERR: DBusError = DBusError::Other("invalid GetLayout response");

impl<L, I, D> HandleReply for GetLayout<L, I, D>
where
    L: GetLayoutList<Item = I>,
    I: GetLayoutItem<List = L>,
    D: AsRef<str> + Clone,
{
    type Output<'a> = L;

    fn handle_reply_body(&self, mut body: IncomingBody<'_>) -> Result<Self::Output<'_>, DBusError> {
        let _ = body.try_next()?.ok_or(ERR)?;
        let root = body.try_next()?.ok_or(ERR)?;
        value_is!(root, IncomingValue::Struct(root));

        let mut iter = root.fields_iter()?;
        let _ = iter.try_next()?.ok_or(ERR)?;
        let _ = iter.try_next()?.ok_or(ERR)?;
        let top_level_items = iter.try_next()?.ok_or(ERR)?;

        value_is!(top_level_items, IncomingValue::Array(top_level_items));

        parse_items(self.destination.as_ref(), &top_level_items)
    }
}

fn parse_items<L, I>(service: &str, items: &IncomingArrayValue<'_>) -> Result<L, DBusError>
where
    L: GetLayoutList<Item = I>,
    I: GetLayoutItem<List = L>,
{
    let mut out = L::new();
    let mut batch = L::new();
    let mut iter = items.items_iter();

    while let Some(item) = iter.try_next()? {
        value_is!(item, IncomingValue::Variant(item));
        let item = item.materialize()?;
        let item = parse_item(service, item)?;
        match item {
            ItemOrSeparator::Skip => {}
            ItemOrSeparator::Item(item) => batch.push(item),
            ItemOrSeparator::Separator => {
                let children = core::mem::take(&mut batch);
                let section = I::new_section(children);
                out.push(section);
            }
        }
    }

    if !batch.is_empty() {
        if out.is_empty() {
            out = batch;
        } else {
            let children = core::mem::take(&mut batch);
            let section = I::new_section(children);
            out.push(section);
        }
    }

    Ok(out)
}

fn parse_item<L, I>(service: &str, item: IncomingValue<'_>) -> Result<ItemOrSeparator<I>, DBusError>
where
    L: GetLayoutList<Item = I>,
    I: GetLayoutItem<List = L>,
{
    value_is!(item, IncomingValue::Struct(fields));

    let mut fields_iter = fields.fields_iter()?;

    let id = fields_iter.try_next()?.ok_or(ERR)?;
    value_is!(id, IncomingValue::Int32(id));

    let props = fields_iter.try_next()?.ok_or(ERR)?;
    value_is!(props, IncomingValue::Array(props));
    let mut props_iter = props.items_iter();
    let mut type_ = "standard";
    let mut label = "";
    let mut enabled = true;
    let mut visible = true;
    let mut toggle_type = "";
    let mut toggle_state = -1;
    let mut children_display = "";
    while let Some(prop) = props_iter.try_next()? {
        value_is!(prop, IncomingValue::DictEntry(dict_entry));
        let (key, value) = dict_entry.key_value()?;
        value_is!(key, IncomingValue::String(key));
        value_is!(value, IncomingValue::Variant(value));

        match key {
            "type" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::String(value));
                type_ = value;
            }
            "label" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::String(value));
                label = value;
            }
            "enabled" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::Bool(value));
                enabled = value;
            }
            "visible" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::Bool(value));
                visible = value;
            }
            "toggle-type" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::String(value));
                toggle_type = value;
            }
            "toggle-state" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::Int32(value));
                toggle_state = value;
            }
            "children-display" => {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::String(value));
                children_display = value;
            }

            _ => {}
        }
    }

    let children_values = fields_iter.try_next()?.ok_or(ERR)?;
    value_is!(children_values, IncomingValue::Array(children_values));
    let children = parse_items::<L, I>(service, &children_values)?;

    if label.len() > 100 {
        label = &label[..100];
    }

    if !visible {
        Ok(ItemOrSeparator::Skip)
    } else if children_display == "submenu" {
        Ok(ItemOrSeparator::Item(I::new_nested(
            id, service, label, children,
        )))
    } else if type_ == "separator" {
        Ok(ItemOrSeparator::Separator)
    } else if !enabled {
        Ok(ItemOrSeparator::Item(I::new_disabled(id, service, label)))
    } else if toggle_type == "checkmark" {
        Ok(ItemOrSeparator::Item(I::new_checkbox(
            id,
            service,
            label,
            toggle_state == 1,
        )))
    } else if toggle_type == "radio" {
        Ok(ItemOrSeparator::Item(I::new_radio(
            id,
            service,
            label,
            toggle_state == 1,
        )))
    } else {
        Ok(ItemOrSeparator::Item(I::new_regular(id, service, label)))
    }
}

#[derive(Debug)]
enum ItemOrSeparator<I> {
    Item(I),
    Separator,
    Skip,
}

/// A list of items built from a part of `GetLayout` reply
pub trait GetLayoutList: Default {
    /// An item type
    type Item: GetLayoutItem;

    /// Constructor
    fn new() -> Self;
    /// Pushes an item
    fn push(&mut self, item: Self::Item);
    /// Returns true if list is empty
    fn is_empty(&self) -> bool;
}

/// A single item build from a part of `GetLayour` reply
pub trait GetLayoutItem {
    /// A list type
    type List: GetLayoutList;

    /// "Section" variant constructor (a group separated from siblings)
    fn new_section(children: Self::List) -> Self;
    /// "Nested" variant constructor (sub-menu)
    fn new_nested(id: i32, service: &str, label: &str, children: Self::List) -> Self;
    /// "Disabled" variant constructor
    fn new_disabled(id: i32, service: &str, label: &str) -> Self;
    /// "Checkbox" variant constructor
    fn new_checkbox(id: i32, service: &str, label: &str, checked: bool) -> Self;
    /// "Radio" variant constructor
    fn new_radio(id: i32, service: &str, label: &str, selected: bool) -> Self;
    /// "Regular" variant constructor (a simple clickable item)
    fn new_regular(id: i32, service: &str, label: &str) -> Self;
}
