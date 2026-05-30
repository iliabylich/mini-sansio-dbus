/// Writes a complete D-Bus message body using const-compatible macro expansion.
///
/// The macro sets the body signature, starts the body, and writes each value. On failure it
/// returns the encoding error from the enclosing function, so that function must return
/// `Result<_, EncodeError>`.
///
/// # Examples
///
/// ```
/// # use mini_sansio_dbus::{dbus_body, MessageType, SliceMessageEncoder};
/// # fn encode(buf: &mut [u8]) -> Result<usize, mini_sansio_dbus::EncodeError> {
/// let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall)?;
/// dbus_body!(encoder, {
///     str("hello"),
///     u32(42),
/// });
/// encoder.finish()
/// # }
/// ```
#[macro_export]
macro_rules! dbus_body {
    ($encoder:expr, { $($body:tt)* }) => {{
        $crate::__dbus_try!($encoder.set_body_signature($crate::__dbus_sig!($($body)*)));
        $crate::__dbus_try!($encoder.__dbus_begin_body());
        $crate::dbus_body_fragment!($encoder, { $($body)* });
    }};
}

/// Writes D-Bus body values without setting or validating the complete body signature.
///
/// This is intended for callbacks that are given an encoder after the surrounding message has
/// already declared its body signature.
#[macro_export]
macro_rules! dbus_body_fragment {
    ($encoder:expr, { $($body:tt)* }) => {{
        $crate::__dbus_write_values!($encoder, $($body)*);
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __dbus_try {
    ($expr:expr) => {
        match $expr {
            Ok(value) => value,
            Err(err) => return Err(err),
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __dbus_sig {
    () => {
        ""
    };
    (u8($value:expr) $(, $($rest:tt)*)?) => {
        concat!("y", $crate::__dbus_sig!($($($rest)*)?))
    };
    (bool($value:expr) $(, $($rest:tt)*)?) => {
        concat!("b", $crate::__dbus_sig!($($($rest)*)?))
    };
    (i16($value:expr) $(, $($rest:tt)*)?) => {
        concat!("n", $crate::__dbus_sig!($($($rest)*)?))
    };
    (u16($value:expr) $(, $($rest:tt)*)?) => {
        concat!("q", $crate::__dbus_sig!($($($rest)*)?))
    };
    (i32($value:expr) $(, $($rest:tt)*)?) => {
        concat!("i", $crate::__dbus_sig!($($($rest)*)?))
    };
    (u32($value:expr) $(, $($rest:tt)*)?) => {
        concat!("u", $crate::__dbus_sig!($($($rest)*)?))
    };
    (i64($value:expr) $(, $($rest:tt)*)?) => {
        concat!("x", $crate::__dbus_sig!($($($rest)*)?))
    };
    (u64($value:expr) $(, $($rest:tt)*)?) => {
        concat!("t", $crate::__dbus_sig!($($($rest)*)?))
    };
    (f64($value:expr) $(, $($rest:tt)*)?) => {
        concat!("d", $crate::__dbus_sig!($($($rest)*)?))
    };
    (unix_fd($value:expr) $(, $($rest:tt)*)?) => {
        concat!("h", $crate::__dbus_sig!($($($rest)*)?))
    };
    (str($value:expr) $(, $($rest:tt)*)?) => {
        concat!("s", $crate::__dbus_sig!($($($rest)*)?))
    };
    (object_path($value:expr) $(, $($rest:tt)*)?) => {
        concat!("o", $crate::__dbus_sig!($($($rest)*)?))
    };
    (signature($value:expr) $(, $($rest:tt)*)?) => {
        concat!("g", $crate::__dbus_sig!($($($rest)*)?))
    };
    (struct_ { $($fields:tt)* } $(, $($rest:tt)*)?) => {
        concat!("(", $crate::__dbus_sig!($($fields)*), ")", $crate::__dbus_sig!($($($rest)*)?))
    };
    (dict_entry { $($fields:tt)* } $(, $($rest:tt)*)?) => {
        concat!("{", $crate::__dbus_sig!($($fields)*), "}", $crate::__dbus_sig!($($($rest)*)?))
    };
    (array<$ty:ident> [$($value:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        concat!("a", $crate::__dbus_type_sig!($ty), $crate::__dbus_sig!($($($rest)*)?))
    };
    (variant<array<$ty:ident>> [$($value:expr),* $(,)?] $(, $($rest:tt)*)?) => {
        concat!("v", $crate::__dbus_sig!($($($rest)*)?))
    };
    (variant<$ty:ident>($value:expr) $(, $($rest:tt)*)?) => {
        concat!("v", $crate::__dbus_sig!($($($rest)*)?))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __dbus_type_sig {
    (u8) => {
        "y"
    };
    (bool) => {
        "b"
    };
    (i16) => {
        "n"
    };
    (u16) => {
        "q"
    };
    (i32) => {
        "i"
    };
    (u32) => {
        "u"
    };
    (i64) => {
        "x"
    };
    (u64) => {
        "t"
    };
    (f64) => {
        "d"
    };
    (unix_fd) => {
        "h"
    };
    (str) => {
        "s"
    };
    (object_path) => {
        "o"
    };
    (signature) => {
        "g"
    };
    (array<$ty:ident>) => {
        concat!("a", $crate::__dbus_type_sig!($ty))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __dbus_type_align {
    (u8) => {
        1
    };
    (bool) => {
        4
    };
    (i16) => {
        2
    };
    (u16) => {
        2
    };
    (i32) => {
        4
    };
    (u32) => {
        4
    };
    (i64) => {
        8
    };
    (u64) => {
        8
    };
    (f64) => {
        8
    };
    (unix_fd) => {
        4
    };
    (str) => {
        4
    };
    (object_path) => {
        4
    };
    (signature) => {
        1
    };
    (array<$ty:ident>) => {
        4
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __dbus_write_values {
    ($encoder:expr,) => {};
    ($encoder:expr) => {};
    ($encoder:expr, $kind:ident($value:expr) $(, $($rest:tt)*)?) => {{
        $crate::__dbus_write_typed_value!($encoder, $kind, $value);
        $crate::__dbus_write_values!($encoder $(, $($rest)*)?);
    }};
    ($encoder:expr, struct_ { $($fields:tt)* } $(, $($rest:tt)*)?) => {{
        $crate::__dbus_try!($encoder.__dbus_align(8));
        $crate::__dbus_write_values!($encoder, $($fields)*);
        $crate::__dbus_write_values!($encoder $(, $($rest)*)?);
    }};
    ($encoder:expr, dict_entry { $($fields:tt)* } $(, $($rest:tt)*)?) => {{
        $crate::__dbus_try!($encoder.__dbus_align(8));
        $crate::__dbus_write_values!($encoder, $($fields)*);
        $crate::__dbus_write_values!($encoder $(, $($rest)*)?);
    }};
    ($encoder:expr, array<$ty:ident> [$($value:expr),* $(,)?] $(, $($rest:tt)*)?) => {{
        $crate::__dbus_write_array!($encoder, $ty, [$($value),*]);
        $crate::__dbus_write_values!($encoder $(, $($rest)*)?);
    }};
    ($encoder:expr, variant<array<$ty:ident>> [$($value:expr),* $(,)?] $(, $($rest:tt)*)?) => {{
        $crate::__dbus_try!(
            $encoder.__dbus_write_signature_value($crate::__dbus_type_sig!(array<$ty>))
        );
        $crate::__dbus_write_array!($encoder, $ty, [$($value),*]);
        $crate::__dbus_write_values!($encoder $(, $($rest)*)?);
    }};
    ($encoder:expr, variant<$ty:ident>($value:expr) $(, $($rest:tt)*)?) => {{
        $crate::__dbus_try!($encoder.__dbus_write_signature_value($crate::__dbus_type_sig!($ty)));
        $crate::__dbus_write_typed_value!($encoder, $ty, $value);
        $crate::__dbus_write_values!($encoder $(, $($rest)*)?);
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __dbus_write_array {
    ($encoder:expr, $ty:ident, [$($value:expr),*]) => {{
        let __dbus_frame =
            $crate::__dbus_try!($encoder.__dbus_start_array($crate::__dbus_type_align!($ty)));
        $(
            $crate::__dbus_write_typed_value!($encoder, $ty, $value);
        )*
        $crate::__dbus_try!($encoder.__dbus_finish_array(__dbus_frame));
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __dbus_write_typed_value {
    ($encoder:expr, u8, $value:expr) => {
        $crate::__dbus_try!($encoder.__dbus_write_u8($value))
    };
    ($encoder:expr, bool, $value:expr) => {
        $crate::__dbus_try!($encoder.__dbus_write_bool($value))
    };
    ($encoder:expr, i16, $value:expr) => {
        $crate::__dbus_try!($encoder.__dbus_write_i16($value))
    };
    ($encoder:expr, u16, $value:expr) => {
        $crate::__dbus_try!($encoder.__dbus_write_u16($value))
    };
    ($encoder:expr, i32, $value:expr) => {
        $crate::__dbus_try!($encoder.__dbus_write_i32($value))
    };
    ($encoder:expr, u32, $value:expr) => {
        $crate::__dbus_try!($encoder.__dbus_write_u32($value))
    };
    ($encoder:expr, i64, $value:expr) => {
        $crate::__dbus_try!($encoder.__dbus_write_i64($value))
    };
    ($encoder:expr, u64, $value:expr) => {
        $crate::__dbus_try!($encoder.__dbus_write_u64($value))
    };
    ($encoder:expr, f64, $value:expr) => {
        $crate::__dbus_try!($encoder.__dbus_write_f64($value))
    };
    ($encoder:expr, unix_fd, $value:expr) => {
        $crate::__dbus_try!($encoder.__dbus_write_unix_fd($value))
    };
    ($encoder:expr, str, $value:expr) => {
        $crate::__dbus_try!($encoder.__dbus_write_string_like($value))
    };
    ($encoder:expr, object_path, $value:expr) => {
        $crate::__dbus_try!($encoder.__dbus_write_string_like($value))
    };
    ($encoder:expr, signature, $value:expr) => {
        $crate::__dbus_try!($encoder.__dbus_write_signature_value($value))
    };
}
