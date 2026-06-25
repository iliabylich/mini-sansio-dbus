#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct Header {
    pub(crate) _endian: u8,
    pub(crate) message_type: u8,
    pub(crate) _flags: u8,
    pub(crate) _protocol_version: u8,
    pub(crate) body_len: u32,
    pub(crate) serial: u32,
    #[expect(clippy::struct_field_names)]
    pub(crate) header_fields_len: u32,
}
