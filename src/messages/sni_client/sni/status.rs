/// SNI item status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusNotifierItemStatus {
    /// The item should not be shown.
    Passive,
    /// The item is active and may be shown.
    Active,
    /// The item requires user attention.
    NeedsAttention,
}

impl StatusNotifierItemStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Passive => "Passive",
            Self::Active => "Active",
            Self::NeedsAttention => "NeedsAttention",
        }
    }
}
