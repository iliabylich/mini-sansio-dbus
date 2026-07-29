use crate::messages::sni_client::sni::{
    IconPixmap, StatusNotifierItemCategory, StatusNotifierItemStatus,
};

/// Caller-provided SNI item state used by `StatusNotifierItemHandler`
pub trait StatusNotifierItemData {
    /// Stable item id
    fn id(&self) -> &str;
    /// User-visible title
    fn title(&self) -> &str;
    /// Current status
    fn status(&self) -> StatusNotifierItemStatus;
    /// Item category
    fn category(&self) -> StatusNotifierItemCategory;
    /// Icon name for theme-based hosts
    fn icon_name(&self) -> &str;
    /// Optional pixmap frame for hosts that prefer raw image data
    fn icon_pixmap(&self) -> Option<IconPixmap<'_>>;
    /// Dbusmenu object path
    fn menu(&self) -> &str;
    /// Whether item activation should open its menu
    fn item_is_menu(&self) -> bool;
}
