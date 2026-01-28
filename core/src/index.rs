// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

/// All types that convert to a usize
///
/// This is used to keep the arena indices small for performance reasons
pub trait Index:
    Into<usize>
    + TryFrom<usize, Error: std::fmt::Debug>
    + std::fmt::Debug
    + Copy
    + Clone
    + std::hash::Hash
    + std::ops::Add<Output = Self>
    + std::ops::AddAssign
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
{
    #[inline]
    fn as_usize(&self) -> usize {
        (*self).into()
    }

    #[inline]
    fn from_usize(n: usize) -> Self {
        Self::try_from(n).unwrap()
    }
}

impl<
    T: Into<usize>
        + TryFrom<usize, Error: std::fmt::Debug>
        + std::fmt::Debug
        + Copy
        + Clone
        + std::hash::Hash
        + std::ops::Add<Output = T>
        + std::ops::AddAssign
        + PartialEq
        + Eq
        + PartialOrd
        + Ord,
> Index for T
{
}
