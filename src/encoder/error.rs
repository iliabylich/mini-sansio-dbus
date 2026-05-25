/// An error produced by the slice-backed message encoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum EncodeError {
    /// The provided output buffer is too small.
    BufferTooSmall,
    /// The message or container signature does not match the value being encoded.
    TypeMismatch,
    /// A string-like value is too long for the D-Bus wire format.
    ValueTooLong,
    /// A container has more bytes than the D-Bus wire format can describe.
    ContainerTooLong,
    /// Header fields cannot be changed after body encoding has started.
    HeaderAlreadyFinished,
    /// The caller tried to encode more body values than declared in the body signature.
    BodySignatureExhausted,
    /// The caller finished the message before encoding all declared body values.
    BodySignatureIncomplete,
    /// A value encoder was used after the value had already been written.
    ValueAlreadyWritten,
}

impl core::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for EncodeError {}

impl EncodeError {
    /// Returns a static string representation of the error.
    #[must_use]
    pub const fn display(self) -> &'static str {
        match self {
            Self::BufferTooSmall => "BufferTooSmall",
            Self::TypeMismatch => "TypeMismatch",
            Self::ValueTooLong => "ValueTooLong",
            Self::ContainerTooLong => "ContainerTooLong",
            Self::HeaderAlreadyFinished => "HeaderAlreadyFinished",
            Self::BodySignatureExhausted => "BodySignatureExhausted",
            Self::BodySignatureIncomplete => "BodySignatureIncomplete",
            Self::ValueAlreadyWritten => "ValueAlreadyWritten",
        }
    }
}
