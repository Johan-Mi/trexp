/// Trait for types that support a monad-like bind operation.
///
/// `FS` is the type of `Self` wrapped in some effect.
pub trait Bind<FS>: Sized {
    /// Binds an effectful [`FnMut`] to an already wrapped value.
    ///
    /// [`FnMut`]: std::ops::FnMut
    fn bind_mut(wrapped: FS, f: impl FnMut(Self) -> FS) -> FS;
}

/// The trivial effect that does nothing, with binding just applying the
/// function directly.
impl<T> Bind<Self> for T {
    fn bind_mut(wrapped: Self, mut f: impl FnMut(Self) -> Self) -> Self {
        f(wrapped)
    }
}

/// [`and_then`] is the monadic bind for [`Result`].
///
/// [`and_then`]: std::result::Result::and_then
/// [`Result`]: std::result::Result
impl<T, E> Bind<Result<Self, E>> for T {
    fn bind_mut(
        wrapped: Result<Self, E>,
        f: impl FnMut(Self) -> Result<Self, E>,
    ) -> Result<Self, E> {
        wrapped.and_then(f)
    }
}
