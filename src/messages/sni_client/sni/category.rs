/// SNI item category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusNotifierItemCategory {
    /// Generic application status.
    ApplicationStatus,
    /// Communications application status.
    Communications,
    /// System service status.
    SystemServices,
    /// Hardware status.
    Hardware,
}

impl StatusNotifierItemCategory {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::ApplicationStatus => "ApplicationStatus",
            Self::Communications => "Communications",
            Self::SystemServices => "SystemServices",
            Self::Hardware => "Hardware",
        }
    }
}
