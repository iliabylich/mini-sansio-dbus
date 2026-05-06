/// a module with `DBus` introspection request/response objects
pub mod introspect;

/// a module with built-in `DBus` request/response objects
pub mod org_freedesktop_dbus;

/// compares given interfaces and returns an error if they are different
#[macro_export]
macro_rules! interface_is {
    ($interface:expr, $expected:expr) => {{
        if $interface != $expected {
            return Err($crate::DBusError::WrongInterface(format!(
                "expected: {:?}, got: {:?}",
                $expected, $interface
            ))
            .into());
        }
    }};
}

/// compares given destinations and returns an error if they are different
#[macro_export]
macro_rules! destination_is {
    ($destination:expr, $expected:expr) => {{
        if $destination != $expected {
            return Err($crate::DBusError::WrongDestination(format!(
                "expected: {:?}, got: {:?}",
                $expected, $destination
            ))
            .into());
        }
    }};
}

/// compares given senders and returns an error if they are different
#[macro_export]
macro_rules! sender_is {
    ($sender:expr, $expected:expr) => {{
        if $sender != $expected {
            return Err($crate::DBusError::WrongSender(format!(
                "expected: {:?}, got: {:?}",
                $expected, $sender
            ))
            .into());
        }
    }};
}

/// compares given paths and returns an error if they are different
#[macro_export]
macro_rules! path_is {
    ($path:expr, $expected:expr) => {{
        if $path != $expected {
            return Err($crate::DBusError::WrongPath(format!(
                "expected: {:?}, got: {:?}",
                $expected, $path
            ))
            .into());
        }
    }};
}

/// compares given members and returns an error if they are different
#[macro_export]
macro_rules! member_is {
    ($member:expr, $expected:expr) => {{
        if $member != $expected {
            return Err($crate::DBusError::WrongMember(format!(
                "expected: {:?}, got: {:?}",
                $expected, $member
            ))
            .into());
        }
    }};
}

/// compares given values and returns an error if they are different
#[macro_export]
macro_rules! value_is {
    ($value:expr, $pat:pat) => {
        let $pat = $value else {
            return Err($crate::DBusError::WrongValue(format!("{:?}", $value)).into());
        };
    };
}
