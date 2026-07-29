use crate::messages::sni_client::dbusmenu::{DBusMenuEvent, DBusMenuItem};

/// Server-side menu state for a `com.canonical.dbusmenu` object
pub trait DBusMenuData {
    /// Root menu list type
    type List: DBusMenuList;

    /// Layout revision. Increment this whenever the menu layout changes
    fn revision(&self) -> u32;

    /// Returns the root menu list
    fn menu(&self) -> &Self::List;

    /// Called for `AboutToShow`; return true when the host should refetch layout
    fn about_to_show(&mut self, _id: i32) -> bool {
        false
    }

    /// Called for dbusmenu `Event` method calls
    fn event(&mut self, _event: DBusMenuEvent<'_>) {}
}

/// A list of dbusmenu items
pub trait DBusMenuList: Sized {
    /// Returns an iterator over the list
    fn iter(&self) -> impl Iterator<Item = &DBusMenuItem<'_, Self>>;
}
