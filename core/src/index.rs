// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

#[allow(unexpected_cfgs, clippy::non_minimal_cfg)]
mod sealed {
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
    fn as_usize(self) -> usize;
    fn from_usize(n: usize) -> Self;
}

#[allow(unexpected_cfgs, clippy::non_minimal_cfg)]
mod impls {
    use super::Index;

    impl Index for usize {
        #[inline(always)]
        fn as_usize(self) -> usize {
            self
        }

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
        #[inline(always)]
        fn as_usize(self) -> usize {
            self as usize
        }

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
        #[inline(always)]
        fn as_usize(self) -> usize {
            self as usize
        }

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
        #[inline(always)]
        fn as_usize(self) -> usize {
            self as usize
        }

        #[inline(always)]
        fn from_usize(n: usize) -> Self {
            n as Self
        }
    }

    #[cfg(any(target_pointer_width = "64", target_pointer_width = "128"))]
    impl Index for u64 {
        #[inline(always)]
        fn as_usize(self) -> usize {
            self as usize
        }

        #[inline(always)]
        fn from_usize(n: usize) -> Self {
            n as Self
        }
    }

    #[cfg(any(target_pointer_width = "128"))]
    impl Index for u128 {
        #[inline(always)]
        fn as_usize(self) -> usize {
            self as usize
        }

        #[inline(always)]
        fn from_usize(n: usize) -> Self {
            n as Self
        }
    }
}
