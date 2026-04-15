use crate::{OutgoingCompleteType, OutgoingSignature, outgoing::EncodingBuffer};

pub(crate) struct SignatureEncoder;

impl SignatureEncoder {
    pub(crate) fn encode_complete_type(
        buf: &mut EncodingBuffer,
        complete_type: &OutgoingCompleteType,
    ) {
        match complete_type {
            OutgoingCompleteType::Byte => buf.encode_u8(b'y'),
            OutgoingCompleteType::Bool => buf.encode_u8(b'b'),
            OutgoingCompleteType::Int16 => buf.encode_u8(b'n'),
            OutgoingCompleteType::UInt16 => buf.encode_u8(b'q'),
            OutgoingCompleteType::Int32 => buf.encode_u8(b'i'),
            OutgoingCompleteType::UInt32 => buf.encode_u8(b'u'),
            OutgoingCompleteType::Int64 => buf.encode_u8(b'x'),
            OutgoingCompleteType::UInt64 => buf.encode_u8(b't'),
            OutgoingCompleteType::Double => buf.encode_u8(b'd'),
            OutgoingCompleteType::UnixFD => buf.encode_u8(b'h'),

            OutgoingCompleteType::String => buf.encode_u8(b's'),
            OutgoingCompleteType::ObjectPath => buf.encode_u8(b'o'),
            OutgoingCompleteType::Signature => buf.encode_u8(b'g'),

            OutgoingCompleteType::Struct(fields) => {
                buf.encode_u8(b'(');
                for field in fields {
                    Self::encode_complete_type(buf, field);
                }
                buf.encode_u8(b')');
            }
            OutgoingCompleteType::Array(item) => {
                buf.encode_u8(b'a');
                Self::encode_complete_type(buf, item);
            }
            OutgoingCompleteType::DictEntry(key, value) => {
                buf.encode_u8(b'{');
                Self::encode_complete_type(buf, key);
                Self::encode_complete_type(buf, value);
                buf.encode_u8(b'}');
            }
            OutgoingCompleteType::Variant => {
                buf.encode_u8(b'v');
            }
        }
    }

    pub(crate) fn encode_signature(buf: &mut EncodingBuffer, signature: &OutgoingSignature) {
        for complete_type in &signature.items {
            Self::encode_complete_type(buf, complete_type);
        }
    }
}
