use crate::{
    Array, DBusError, DictEntry, IncomingValue, MessageType, ObjectPath, OutgoingCompleteType,
    OutgoingMessage, OutgoingValue, Signature, SliceMessageEncoder, Str, Struct2, UnixFd, Variant,
    outgoing::MessageEncoder,
};

const MESSAGE_BLOB: &[u8] = &[
    108, 1, 0, 1, 156, 0, 0, 0, 42, 0, 0, 0, 174, 0, 0, 0, 1, 1, 111, 0, 19, 0, 0, 0, 47, 111, 114,
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

fn message_with_known_value_types() -> OutgoingMessage {
    OutgoingMessage::MethodCall {
        destination: Some(String::from("org.example.Service")),
        path: String::from("/org/example/Object"),
        interface: Some(String::from("org.example.Interface")),
        serial: 42,
        member: String::from("AllTypes"),
        sender: Some(String::from(":1.100")),
        unix_fds: Some(1),
        body: vec![
            OutgoingValue::Byte(0x2a),
            OutgoingValue::Bool(true),
            OutgoingValue::Int16(-1234),
            OutgoingValue::UInt16(1234),
            OutgoingValue::Int32(-123_456),
            OutgoingValue::UInt32(123_456),
            OutgoingValue::Int64(-123_456_789),
            OutgoingValue::UInt64(123_456_789),
            OutgoingValue::Double(12.5),
            OutgoingValue::UnixFD(0),
            OutgoingValue::String(String::from("hello")),
            OutgoingValue::ObjectPath(String::from("/org/example/Value")),
            OutgoingValue::Signature(b"su".to_vec()),
            OutgoingValue::Struct(vec![
                OutgoingValue::String(String::from("inside-struct")),
                OutgoingValue::UInt32(77),
            ]),
            OutgoingValue::Array(
                OutgoingCompleteType::UInt16,
                vec![OutgoingValue::UInt16(7), OutgoingValue::UInt16(8)],
            ),
            OutgoingValue::DictEntry(
                Box::new(OutgoingValue::String(String::from("dict-key"))),
                Box::new(OutgoingValue::UInt32(99)),
            ),
            OutgoingValue::Variant(Box::new(OutgoingValue::Int32(-9))),
        ],
    }
}

fn encoded_message_blob() -> Vec<u8> {
    MESSAGE_BLOB.to_vec()
}

#[test]
fn slice_encoder_encodes_message_to_expected_in_memory_blob() -> Result<(), crate::EncodeError> {
    let mut buf = vec![0; 512];
    let mut encoder = SliceMessageEncoder::new(&mut buf, MessageType::MethodCall, 42)?;
    encoder.set_path("/org/example/Object")?;
    encoder.set_interface("org.example.Interface")?;
    encoder.set_member("AllTypes")?;
    encoder.set_destination("org.example.Service")?;
    encoder.set_sender(":1.100")?;
    encoder.set_unix_fds(1)?;
    encoder.set_body_signature("ybnqiuxtdhsog(su)aq{su}v")?;

    encoder.next_body_slot::<u8>()?.write(0x2a)?;
    encoder.next_body_slot::<bool>()?.write(true)?;
    encoder.next_body_slot::<i16>()?.write(-1234)?;
    encoder.next_body_slot::<u16>()?.write(1234)?;
    encoder.next_body_slot::<i32>()?.write(-123_456)?;
    encoder.next_body_slot::<u32>()?.write(123_456)?;
    encoder.next_body_slot::<i64>()?.write(-123_456_789)?;
    encoder.next_body_slot::<u64>()?.write(123_456_789)?;
    encoder.next_body_slot::<f64>()?.write(12.5)?;
    encoder.next_body_slot::<UnixFd>()?.write(0)?;
    encoder.next_body_slot::<Str>()?.write("hello")?;
    encoder
        .next_body_slot::<ObjectPath>()?
        .write("/org/example/Value")?;
    encoder.next_body_slot::<Signature>()?.write("su")?;
    {
        let mut slot = encoder.next_body_slot::<Struct2<Str, u32>>()?;
        slot.first_slot()?.write("inside-struct")?;
        slot.second_slot()?.write(77)?;
    }
    {
        let mut array = encoder.next_body_slot::<Array<u16>>()?;
        array.next_slot()?.write(7)?;
        array.next_slot()?.write(8)?;
    }
    {
        let mut slot = encoder.next_body_slot::<DictEntry<Str, u32>>()?;
        slot.key_slot()?.write("dict-key")?;
        slot.value_slot()?.write(99)?;
    }
    {
        let mut slot = encoder.next_body_slot::<Variant<i32>>()?;
        slot.payload_slot()?.write(-9)?;
    }
    let len = encoder.finish()?;

    assert_eq!(&buf[..len], MESSAGE_BLOB);

    Ok(())
}

#[test]
fn encodes_message_to_expected_in_memory_blob() {
    let encoded = MessageEncoder::encode(&message_with_known_value_types());

    assert_eq!(encoded, encoded_message_blob());
}

#[test]
fn decodes_message_from_same_in_memory_blob() -> Result<(), DBusError> {
    let encoded = encoded_message_blob();
    let decoded = crate::IncomingMessage::new(&encoded)?;

    assert_eq!(decoded.message_type, MessageType::MethodCall);
    assert_eq!(decoded.serial, 42);
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
        panic!("expected struct value");
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
        panic!("expected array value");
    };
    let mut items = array.items_iter();
    assert!(matches!(items.try_next()?, Some(IncomingValue::UInt16(7))));
    assert!(matches!(items.try_next()?, Some(IncomingValue::UInt16(8))));
    assert!(items.try_next()?.is_none());

    let Some(IncomingValue::DictEntry(dict_entry)) = body.try_next()? else {
        panic!("expected dict entry value");
    };
    let (key, value) = dict_entry.key_value()?;
    assert!(matches!(key, IncomingValue::String("dict-key")));
    assert!(matches!(value, IncomingValue::UInt32(99)));

    let Some(IncomingValue::Variant(variant)) = body.try_next()? else {
        panic!("expected variant value");
    };
    assert!(matches!(variant.materialize()?, IncomingValue::Int32(-9)));
    assert!(body.try_next()?.is_none());

    Ok(())
}
