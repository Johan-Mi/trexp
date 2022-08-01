pub use Rewrite::{Clean, Dirty};

use crate::Bind;
use std::ops::{Deref, DerefMut};

/// Enum representing a value that has passed through a transformation that may
/// or may not have affected it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rewrite<T> {
    /// The contained value *was not* affected by the transformation.
    Clean(T),
    /// The contained value *was* affected by the transformation.
    Dirty(T),
}

impl<T> Rewrite<T> {
    /// Returns `true` if the rewrite is [`Clean`].
    ///
    /// [`Clean`]: Rewrite::Clean
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        matches!(self, Clean(..))
    }

    /// Returns `true` if the rewrite is [`Dirty`].
    ///
    /// [`Dirty`]: Rewrite::Dirty
    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        matches!(self, Dirty(..))
    }

    /// Takes the contained value, forgetting whether it's [`Clean`] or
    /// [`Dirty`].
    ///
    /// [`Clean`]: Rewrite::Clean
    /// [`Dirty`]: Rewrite::Dirty
    #[allow(clippy::missing_const_for_fn)] // requires const_precise_live_drops
    pub fn into_inner(self) -> T {
        match self {
            Clean(clean) => clean,
            Dirty(dirty) => dirty,
        }
    }

    /// Borrows the inner value.
    pub const fn inner_ref(&self) -> &T {
        let (Clean(t) | Dirty(t)) = self;
        t
    }

    /// Mutably borrows the inner value.
    pub fn inner_mut(&mut self) -> &mut T {
        let (Clean(t) | Dirty(t)) = self;
        t
    }

    /// Converts from `&Rewrite<T>` to `Rewrite<&T>`.
    pub const fn as_ref(&self) -> Rewrite<&T> {
        match self {
            Clean(t) => Clean(t),
            Dirty(t) => Dirty(t),
        }
    }

    /// Converts from `&mut Rewrite<T>` to `Rewrite<&mut T>`.
    pub fn as_mut(&mut self) -> Rewrite<&mut T> {
        match self {
            Clean(t) => Clean(t),
            Dirty(t) => Dirty(t),
        }
    }

    /// Converts from `&Rewrite<T>` to `Rewrite<&T::Target>`.
    pub fn as_deref(&self) -> Rewrite<&T::Target>
    where
        T: Deref,
    {
        match self {
            Clean(t) => Clean(t),
            Dirty(t) => Dirty(t),
        }
    }

    /// Converts from `&mut Rewrite<T>` to `Rewrite<&mut T::Target>`.
    pub fn as_deref_mut(&mut self) -> Rewrite<&T::Target>
    where
        T: DerefMut,
    {
        match self {
            Clean(t) => Clean(t),
            Dirty(t) => Dirty(t),
        }
    }

    /// Maps a function over `self`, retaining whether it's [`Clean`] or
    /// [`Dirty`].
    ///
    /// [`Clean`]: Rewrite::Clean
    /// [`Dirty`]: Rewrite::Dirty
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Rewrite<U> {
        match self {
            Clean(t) => Clean(f(t)),
            Dirty(t) => Dirty(f(t)),
        }
    }

    /// Repeatedly applies a function until its result is [`Clean`].
    ///
    /// [`Clean`]: Rewrite::Clean
    ///
    /// # Examples
    ///
    /// ```
    /// # use trexp::{Rewrite, Clean, Dirty};
    /// fn halve_even(n: i32) -> Rewrite<i32> {
    ///     if n % 2 == 0 {
    ///         Dirty(n / 2)
    ///     } else {
    ///         Clean(n)
    ///     }
    /// }
    ///
    /// let halve_repeatedly = |n| Rewrite::repeat(n, halve_even);
    ///
    /// assert_eq!(Dirty(5), halve_repeatedly(10)); // Succeeded once
    /// assert_eq!(Dirty(3), halve_repeatedly(24)); // Succeeded thrice
    /// assert_eq!(Clean(7), halve_repeatedly(7)); // Didn't succeed
    /// ```
    pub fn repeat(initial: T, mut f: impl FnMut(T) -> Self) -> Self {
        let mut val = Clean(initial);
        loop {
            match val.map(&mut f).transpose() {
                Clean(done) => break done,
                Dirty(keep_going) => val = Dirty(keep_going.into_inner()),
            }
        }
    }

    /// A version of [`repeat`] that takes a fallible function.
    ///
    /// [`repeat`]: Rewrite::repeat
    pub fn try_repeat<E>(
        initial: T,
        mut f: impl FnMut(T) -> Result<Self, E>,
    ) -> Result<Self, E> {
        let mut val = Clean(initial);
        loop {
            match val.map(&mut f).transpose_result()?.transpose() {
                Clean(done) => break Ok(done),
                Dirty(keep_going) => val = Dirty(keep_going.into_inner()),
            }
        }
    }

    /// Applies a function and makes the result [`Dirty`] if `self` was already
    /// dirty or became dirty as a result of the function.
    ///
    /// [`Dirty`]: Rewrite::Dirty
    pub fn bind(self, f: impl FnOnce(T) -> Self) -> Self {
        match self {
            Clean(clean) => f(clean),
            Dirty(dirty) => Dirty(f(dirty).into_inner()),
        }
    }

    /// A version of [`bind`] that takes a fallible function.
    ///
    /// [`bind`]: Rewrite::bind
    pub fn try_bind<E>(
        self,
        f: impl FnOnce(T) -> Result<Self, E>,
    ) -> Result<Self, E> {
        match self {
            Clean(t) => f(t),
            Dirty(t) => Ok(Dirty(f(t)?.into_inner())),
        }
    }
}

impl<T> Rewrite<&T> {
    /// Creates a new [`Rewrite`] by copying the inner value.
    ///
    /// [`Rewrite`]: Rewrite
    pub const fn copied(self) -> Rewrite<T>
    where
        T: Copy,
    {
        match self {
            Clean(t) => Clean(*t),
            Dirty(t) => Dirty(*t),
        }
    }

    /// Creates a new [`Rewrite`] by cloning the inner value.
    ///
    /// [`Rewrite`]: Rewrite
    pub fn cloned(self) -> Rewrite<T>
    where
        T: Clone,
    {
        self.map(Clone::clone)
    }
}

impl<T> Rewrite<Rewrite<T>> {
    /// Swaps two layers of [`Rewrite`].
    ///
    /// [`Rewrite`]: Rewrite
    ///
    /// # Examples
    ///
    /// ```
    /// # use trexp::{Clean, Dirty};
    /// assert_eq!(Dirty(Clean(42)), Clean(Dirty(42)).transpose());
    /// assert_eq!(Clean(Dirty(42)), Dirty(Clean(42)).transpose());
    /// ```
    pub fn transpose(self) -> Self {
        match self {
            Clean(inner) => inner.map(Clean),
            Dirty(inner) => inner.map(Dirty),
        }
    }
}

impl<T, E> Rewrite<Result<T, E>> {
    /// Converts `Rewrite<Result<T, E>>` into `Result<Rewrite<T, E>>`.
    pub fn transpose_result(self) -> Result<Rewrite<T>, E> {
        match self {
            Clean(t) => Ok(Clean(t?)),
            Dirty(t) => Ok(Dirty(t?)),
        }
    }
}

/// Makes the entire collection [`Dirty`] if any of the elements are.
///
/// [`Dirty`]: Rewrite::Dirty
impl<T, C> FromIterator<Rewrite<T>> for Rewrite<C>
where
    C: FromIterator<T>,
{
    fn from_iter<I: IntoIterator<Item = Rewrite<T>>>(iter: I) -> Self {
        let mut is_dirty = false;
        let collected = iter
            .into_iter()
            .inspect(|item| is_dirty |= item.is_dirty())
            .map(Rewrite::into_inner)
            .collect();
        if is_dirty {
            Dirty(collected)
        } else {
            Clean(collected)
        }
    }
}

impl<T> Bind<Rewrite<Self>> for T {
    fn bind_mut(
        wrapped: Rewrite<Self>,
        f: impl FnMut(Self) -> Rewrite<Self>,
    ) -> Rewrite<Self> {
        wrapped.bind(f)
    }
}
