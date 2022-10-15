use crate::Rewrite;

/// Trait for types that support a monad-like bind operation.
///
/// `FS` is the type of `Self` wrapped in some effect.
pub trait Bind<FS>: Sized {
    /// Binds an effectful [`FnMut`] to an already wrapped value.
    ///
    /// [`FnMut`]: core::ops::FnMut
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
/// [`and_then`]: core::result::Result::and_then
/// [`Result`]: core::result::Result
impl<T, E> Bind<Result<Self, E>> for T {
    fn bind_mut(
        wrapped: Result<Self, E>,
        f: impl FnMut(Self) -> Result<Self, E>,
    ) -> Result<Self, E> {
        wrapped.and_then(f)
    }
}

/// The effect stack consisting of both [`Result`] and [`Rewrite`].
///
/// [`Result`]: core::result::Result
/// [`Rewrite`]: crate::Rewrite
impl<T, E> Bind<Result<Rewrite<Self>, E>> for T {
    fn bind_mut(
        wrapped: Result<Rewrite<Self>, E>,
        f: impl FnMut(Self) -> Result<Rewrite<Self>, E>,
    ) -> Result<Rewrite<Self>, E> {
        wrapped?.try_bind(f)
    }
}
