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
