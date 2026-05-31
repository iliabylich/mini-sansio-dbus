use core::marker::PhantomData;

/// Either constant or dynamic value for Path / Interface / Property name
pub enum Conf<V: ?Sized + 'static, This: ?Sized> {
    #[doc(hidden)]
    Constant(&'static V),
    #[doc(hidden)]
    Dynamic {
        f: for<'a> fn(&'a This) -> &'a V,
        _phantom: PhantomData<This>,
    },
}
impl<V: ?Sized + 'static, This: ?Sized> Conf<V, This> {
    /// Constructs constant variant
    pub const fn constant(value: &'static V) -> Self {
        Self::Constant(value)
    }

    /// Constructs dynamic variant
    pub const fn dynamic(f: for<'a> fn(&'a This) -> &'a V) -> Self {
        Self::Dynamic {
            f,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn resolve<'a>(&self, this: &'a This) -> &'a V {
        match self {
            Self::Constant(v) => v,
            Self::Dynamic { f, _phantom } => (f)(this),
        }
    }
}
