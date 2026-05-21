/// a module with `DBus` introspection request/response objects
pub mod introspect;

/// a module with built-in `DBus` request/response objects
pub mod org_freedesktop_dbus;

/// compares given interfaces and returns an error if they are different
#[macro_export]
macro_rules! interface_is {
    ($interface:expr, $expected:expr) => {{
        if $interface != $expected {
            return Err($crate::DBusError::WrongInterface.into());
        }
    }};
}

/// compares given destinations and returns an error if they are different
#[macro_export]
macro_rules! destination_is {
    ($destination:expr, $expected:expr) => {{
        if $destination != $expected {
            return Err($crate::DBusError::WrongDestination.into());
        }
    }};
}

/// compares given senders and returns an error if they are different
#[macro_export]
macro_rules! sender_is {
    ($sender:expr, $expected:expr) => {{
        if $sender != $expected {
            return Err($crate::DBusError::WrongSender.into());
        }
    }};
}

/// compares given paths and returns an error if they are different
#[macro_export]
macro_rules! path_is {
    ($path:expr, $expected:expr) => {{
        if $path != $expected {
            return Err($crate::DBusError::WrongPath.into());
        }
    }};
}

/// compares given members and returns an error if they are different
#[macro_export]
macro_rules! member_is {
    ($member:expr, $expected:expr) => {{
        if $member != $expected {
            return Err($crate::DBusError::WrongMember.into());
        }
    }};
}

/// compares given values and returns an error if they are different
#[macro_export]
macro_rules! value_is {
    ($value:expr, $pat:pat) => {
        let $pat = $value else {
            return Err($crate::DBusError::WrongValue.into());
        };
    };
}
