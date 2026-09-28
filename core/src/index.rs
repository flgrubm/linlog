// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

/// Keeps `Index` implementable only in this crate.
#[allow(unexpected_cfgs, clippy::non_minimal_cfg)]
mod sealed {
    /// Implemented exactly by the types that may be an `Index`.
    pub trait Sealed {}

    impl Sealed for usize {}

    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64",
        target_pointer_width = "128"
    ))]
    impl Sealed for u8 {}

    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64",
        target_pointer_width = "128"
    ))]
    impl Sealed for u16 {}

    #[cfg(any(
        target_pointer_width = "32",
        target_pointer_width = "64",
        target_pointer_width = "128"
    ))]
    impl Sealed for u32 {}

    #[cfg(any(target_pointer_width = "64", target_pointer_width = "128"))]
    impl Sealed for u64 {}

    #[cfg(any(target_pointer_width = "128"))]
    impl Sealed for u128 {}
}

use sealed::Sealed;

/// All types that convert to a usize
///
/// This is used to keep the arena indices small for performance reasons
pub trait Index:
    Sealed
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
    /// Returns the index as a `usize`.
    fn as_usize(self) -> usize;

    /// Returns `n` as this index type, truncated if it does not fit.
    fn from_usize(n: usize) -> Self;
}

/// `Index` for every unsigned integer type no wider than `usize`.
#[allow(unexpected_cfgs, clippy::non_minimal_cfg)]
mod impls {
    use super::Index;

    impl Index for usize {
        /// Returns the index as a `usize`.
        #[inline(always)]
        fn as_usize(self) -> usize {
            self
        }

        /// Returns `n` as this index type, truncated if it does not fit.
        #[inline(always)]
        fn from_usize(n: usize) -> Self {
            n as Self
        }
    }

    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64",
        target_pointer_width = "128"
    ))]
    impl Index for u8 {
        /// Returns the index as a `usize`.
        #[inline(always)]
        fn as_usize(self) -> usize {
            self as usize
        }

        /// Returns `n` as this index type, truncated if it does not fit.
        #[inline(always)]
        fn from_usize(n: usize) -> Self {
            n as Self
        }
    }

    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64",
        target_pointer_width = "128"
    ))]
    impl Index for u16 {
        /// Returns the index as a `usize`.
        #[inline(always)]
        fn as_usize(self) -> usize {
            self as usize
        }

        /// Returns `n` as this index type, truncated if it does not fit.
        #[inline(always)]
        fn from_usize(n: usize) -> Self {
            n as Self
        }
    }

    #[cfg(any(
        target_pointer_width = "32",
        target_pointer_width = "64",
        target_pointer_width = "128"
    ))]
    impl Index for u32 {
        /// Returns the index as a `usize`.
        #[inline(always)]
        fn as_usize(self) -> usize {
            self as usize
        }

        /// Returns `n` as this index type, truncated if it does not fit.
        #[inline(always)]
        fn from_usize(n: usize) -> Self {
            n as Self
        }
    }

    #[cfg(any(target_pointer_width = "64", target_pointer_width = "128"))]
    impl Index for u64 {
        /// Returns the index as a `usize`.
        #[inline(always)]
        fn as_usize(self) -> usize {
            self as usize
        }

        /// Returns `n` as this index type, truncated if it does not fit.
        #[inline(always)]
        fn from_usize(n: usize) -> Self {
            n as Self
        }
    }

    #[cfg(any(target_pointer_width = "128"))]
    impl Index for u128 {
        /// Returns the index as a `usize`.
        #[inline(always)]
        fn as_usize(self) -> usize {
            self as usize
        }

        /// Returns `n` as this index type, truncated if it does not fit.
        #[inline(always)]
        fn from_usize(n: usize) -> Self {
            n as Self
        }
    }
}
