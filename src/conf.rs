use core::marker::PhantomData;

/// Either constant or dynamic value for Path / Interface / Property name
#[derive(Clone, Copy)]
pub enum Conf<V: ?Sized + 'static, Data: ?Sized> {
    #[doc(hidden)]
    Constant(&'static V),
    #[doc(hidden)]
    Dynamic {
        f: for<'a> fn(&'a Data) -> &'a V,
        _phantom: PhantomData<Data>,
    },
}
impl<V: ?Sized + 'static, Data: ?Sized> Conf<V, Data> {
    /// Constructs constant variant
    pub const fn constant(value: &'static V) -> Self {
        Self::Constant(value)
    }

    /// Constructs dynamic variant
    pub const fn dynamic(f: for<'a> fn(&'a Data) -> &'a V) -> Self {
        Self::Dynamic {
            f,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn resolve(self, data: &Data) -> &V {
        match self {
            Self::Constant(v) => v,
            Self::Dynamic { f, _phantom } => (f)(data),
        }
    }
}
