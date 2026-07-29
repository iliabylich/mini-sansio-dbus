use crate::{
    DBusError, EncodeError, IncomingMessage, IncomingValue, MessageType, SliceMessageEncoder,
    dbus_body, dbus_body_fragment, messages::org_freedesktop_dbus::SetProperty,
    messaging::DBusEncode,
};

const MESSAGE_BLOB: &[u8] = &[
    108, 1, 0, 1, 156, 0, 0, 0, 0, 0, 0, 0, 174, 0, 0, 0, 1, 1, 111, 0, 19, 0, 0, 0, 47, 111, 114,
    103, 47, 101, 120, 97, 109, 112, 108, 101, 47, 79, 98, 106, 101, 99, 116, 0, 0, 0, 0, 0, 2, 1,
    115, 0, 21, 0, 0, 0, 111, 114, 103, 46, 101, 120, 97, 109, 112, 108, 101, 46, 73, 110, 116,
    101, 114, 102, 97, 99, 101, 0, 0, 0, 3, 1, 115, 0, 8, 0, 0, 0, 65, 108, 108, 84, 121, 112, 101,
    115, 0, 0, 0, 0, 0, 0, 0, 0, 6, 1, 115, 0, 19, 0, 0, 0, 111, 114, 103, 46, 101, 120, 97, 109,
    112, 108, 101, 46, 83, 101, 114, 118, 105, 99, 101, 0, 0, 0, 0, 0, 7, 1, 115, 0, 6, 0, 0, 0,
    58, 49, 46, 49, 48, 48, 0, 0, 9, 1, 117, 0, 1, 0, 0, 0, 8, 1, 103, 0, 24, 121, 98, 110, 113,
    105, 117, 120, 116, 100, 104, 115, 111, 103, 40, 115, 117, 41, 97, 113, 123, 115, 117, 125,
    118, 0, 0, 0, 42, 0, 0, 0, 1, 0, 0, 0, 46, 251, 210, 4, 192, 29, 254, 255, 64, 226, 1, 0, 0, 0,
    0, 0, 235, 50, 164, 248, 255, 255, 255, 255, 21, 205, 91, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 41,
    64, 0, 0, 0, 0, 5, 0, 0, 0, 104, 101, 108, 108, 111, 0, 0, 0, 18, 0, 0, 0, 47, 111, 114, 103,
    47, 101, 120, 97, 109, 112, 108, 101, 47, 86, 97, 108, 117, 101, 0, 2, 115, 117, 0, 0, 0, 0, 0,
    0, 13, 0, 0, 0, 105, 110, 115, 105, 100, 101, 45, 115, 116, 114, 117, 99, 116, 0, 0, 0, 77, 0,
    0, 0, 4, 0, 0, 0, 7, 0, 8, 0, 8, 0, 0, 0, 100, 105, 99, 116, 45, 107, 101, 121, 0, 0, 0, 0, 99,
    0, 0, 0, 1, 105, 0, 0, 247, 255, 255, 255,
];

fn encode_message() -> Result<([u8; 128], usize), EncodeError> {
    let mut buf = [0; 128];
    let mut encoder = SliceMessageEncoder::new(&mut buf, MessageType::MethodCall)?;
    dbus_body!(encoder, {
        u32(42),
        str("const"),
        array<u16> [1, 2],
    });
    let len = encoder.finish()?;
    Ok((buf, len))
}

#[test]
fn encoder_encodes_message_to_expected_in_memory_blob() -> Result<(), EncodeError> {
    let mut buf = [0; 512];
    let mut encoder = SliceMessageEncoder::new(&mut buf, MessageType::MethodCall)?;
    encoder.set_path("/org/example/Object")?;
    encoder.set_interface("org.example.Interface")?;
    encoder.set_member("AllTypes")?;
    encoder.set_destination("org.example.Service")?;
    encoder.set_sender(":1.100")?;
    encoder.set_unix_fds(1)?;

    dbus_body!(encoder, {
        u8(0x2a),
        bool(true),
        i16(-1234),
        u16(1234),
        i32(-123_456),
        u32(123_456),
        i64(-123_456_789),
        u64(123_456_789),
        f64(12.5),
        unix_fd(0),
        str("hello"),
        object_path("/org/example/Value"),
        signature("su"),
        struct_ {
            str("inside-struct"),
            u32(77),
        },
        array<u16> [7, 8],
        dict_entry {
            str("dict-key"),
            u32(99),
        },
        variant<i32>(-9),
    });

    let len = encoder.finish()?;

    assert_eq!(
        buf.get(..len).ok_or(EncodeError::BufferTooSmall)?,
        MESSAGE_BLOB
    );

    Ok(())
}

#[test]
fn macro_encoder_can_run_in_const_context() -> Result<(), DBusError> {
    let Ok((buf, len)) = encode_message() else {
        return Err(DBusError::MalformedBody);
    };
    let message = IncomingMessage::new(buf.get(..len).ok_or(DBusError::MalformedBody)?)?;

    assert_eq!(message.serial, 0);
    assert_eq!(message.signature, Some("usaq"));

    let mut body = message.body.ok_or(DBusError::MalformedBody)?;
    assert!(matches!(body.try_next()?, Some(IncomingValue::UInt32(42))));
    assert!(matches!(
        body.try_next()?,
        Some(IncomingValue::String("const"))
    ));

    let Some(IncomingValue::Array(array)) = body.try_next()? else {
        return Err(DBusError::WrongValue);
    };
    let mut items = array.items_iter();
    assert!(matches!(items.try_next()?, Some(IncomingValue::UInt16(1))));
    assert!(matches!(items.try_next()?, Some(IncomingValue::UInt16(2))));
    assert!(items.try_next()?.is_none());
    assert!(body.try_next()?.is_none());

    Ok(())
}

#[test]
fn decodes_message_from_same_in_memory_blob() -> Result<(), DBusError> {
    let decoded = IncomingMessage::new(MESSAGE_BLOB)?;

    assert_eq!(decoded.message_type, MessageType::MethodCall);
    assert_eq!(decoded.serial, 0);
    assert_eq!(decoded.destination, Some("org.example.Service"));
    assert_eq!(decoded.path, Some("/org/example/Object"));
    assert_eq!(decoded.interface, Some("org.example.Interface"));
    assert_eq!(decoded.member, Some("AllTypes"));
    assert_eq!(decoded.sender, Some(":1.100"));
    assert_eq!(decoded.signature, Some("ybnqiuxtdhsog(su)aq{su}v"));
    assert_eq!(decoded.unix_fds, Some(1));

    let mut body = decoded.body.ok_or(DBusError::MalformedBody)?;
    assert!(matches!(body.try_next()?, Some(IncomingValue::Byte(0x2a))));
    assert!(matches!(body.try_next()?, Some(IncomingValue::Bool(true))));
    assert!(matches!(
        body.try_next()?,
        Some(IncomingValue::Int16(-1234))
    ));
    assert!(matches!(
        body.try_next()?,
        Some(IncomingValue::UInt16(1234))
    ));
    assert!(matches!(
        body.try_next()?,
        Some(IncomingValue::Int32(-123_456))
    ));
    assert!(matches!(
        body.try_next()?,
        Some(IncomingValue::UInt32(123_456))
    ));
    assert!(matches!(
        body.try_next()?,
        Some(IncomingValue::Int64(-123_456_789))
    ));
    assert!(matches!(
        body.try_next()?,
        Some(IncomingValue::UInt64(123_456_789))
    ));
    assert!(matches!(
        body.try_next()?,
        Some(IncomingValue::Double(12.5))
    ));
    assert!(matches!(body.try_next()?, Some(IncomingValue::UnixFD(0))));
    assert!(matches!(
        body.try_next()?,
        Some(IncomingValue::String("hello"))
    ));
    assert!(matches!(
        body.try_next()?,
        Some(IncomingValue::ObjectPath("/org/example/Value"))
    ));
    assert!(matches!(
        body.try_next()?,
        Some(IncomingValue::Signature("su"))
    ));

    let Some(IncomingValue::Struct(struct_)) = body.try_next()? else {
        return Err(DBusError::WrongValue);
    };
    let mut fields = struct_.fields_iter()?;
    assert!(matches!(
        fields.try_next()?,
        Some(IncomingValue::String("inside-struct"))
    ));
    assert!(matches!(
        fields.try_next()?,
        Some(IncomingValue::UInt32(77))
    ));
    assert!(fields.try_next()?.is_none());

    let Some(IncomingValue::Array(array)) = body.try_next()? else {
        return Err(DBusError::WrongValue);
    };
    let mut items = array.items_iter();
    assert!(matches!(items.try_next()?, Some(IncomingValue::UInt16(7))));
    assert!(matches!(items.try_next()?, Some(IncomingValue::UInt16(8))));
    assert!(items.try_next()?.is_none());

    let Some(IncomingValue::DictEntry(dict_entry)) = body.try_next()? else {
        return Err(DBusError::WrongValue);
    };
    let (key, value) = dict_entry.key_value()?;
    assert!(matches!(key, IncomingValue::String("dict-key")));
    assert!(matches!(value, IncomingValue::UInt32(99)));

    let Some(IncomingValue::Variant(variant)) = body.try_next()? else {
        return Err(DBusError::WrongValue);
    };
    assert!(matches!(variant.materialize()?, IncomingValue::Int32(-9)));
    assert!(body.try_next()?.is_none());

    Ok(())
}

#[test]
fn set_property_encodes_string_variant() -> Result<(), DBusError> {
    let mut buf = [0; 512];
    let write_value = |encoder: &mut SliceMessageEncoder<'_>| {
        dbus_body_fragment!(encoder, {
            variant<str>("online"),
        });
        Ok(())
    };
    let buf = SetProperty::encode(
        (
            "org.example.Service",
            "/org/example/Object",
            "org.example.Interface",
            "Name",
            &write_value,
        ),
        &mut buf,
    )?;
    let decoded = IncomingMessage::new(buf)?;

    assert_eq!(decoded.message_type, MessageType::MethodCall);
    assert_eq!(decoded.serial, 0);
    assert_eq!(decoded.destination, Some("org.example.Service"));
    assert_eq!(decoded.path, Some("/org/example/Object"));
    assert_eq!(decoded.interface, Some("org.freedesktop.DBus.Properties"));
    assert_eq!(decoded.member, Some("Set"));
    assert_eq!(decoded.signature, Some("ssv"));

    let mut body = decoded.body.ok_or(DBusError::MalformedBody)?;
    assert!(matches!(
        body.try_next()?,
        Some(IncomingValue::String("org.example.Interface"))
    ));
    assert!(matches!(
        body.try_next()?,
        Some(IncomingValue::String("Name"))
    ));
    let Some(IncomingValue::Variant(variant)) = body.try_next()? else {
        return Err(DBusError::WrongValue);
    };
    assert!(matches!(
        variant.materialize()?,
        IncomingValue::String("online")
    ));
    assert!(body.try_next()?.is_none());

    Ok(())
}

#[test]
fn set_property_encodes_array_variant() -> Result<(), DBusError> {
    let mut buf = [0; 512];
    let values = [1u32, 2, 3];
    let write_value = |encoder: &mut SliceMessageEncoder<'_>| {
        dbus_body_fragment!(encoder, {
            variant<array<u32>> [values[0], values[1], values[2]],
        });
        Ok(())
    };
    let buf = SetProperty::encode(
        (
            "org.example.Service",
            "/org/example/Object",
            "org.example.Interface",
            "Values",
            &write_value,
        ),
        &mut buf,
    )?;
    let decoded = IncomingMessage::new(buf)?;

    assert_eq!(decoded.signature, Some("ssv"));

    let mut body = decoded.body.ok_or(DBusError::MalformedBody)?;
    assert!(matches!(
        body.try_next()?,
        Some(IncomingValue::String("org.example.Interface"))
    ));
    assert!(matches!(
        body.try_next()?,
        Some(IncomingValue::String("Values"))
    ));
    let Some(IncomingValue::Variant(variant)) = body.try_next()? else {
        return Err(DBusError::WrongValue);
    };
    let IncomingValue::Array(array) = variant.materialize()? else {
        return Err(DBusError::WrongValue);
    };
    let mut items = array.items_iter();
    assert!(matches!(items.try_next()?, Some(IncomingValue::UInt32(1))));
    assert!(matches!(items.try_next()?, Some(IncomingValue::UInt32(2))));
    assert!(matches!(items.try_next()?, Some(IncomingValue::UInt32(3))));
    assert!(items.try_next()?.is_none());
    assert!(body.try_next()?.is_none());

    Ok(())
}
