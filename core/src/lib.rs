// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

#![allow(dead_code)]
#![allow(unused_variables)]

mod errors;
pub mod index;
pub mod linear;
pub mod logics;
#[cfg(feature = "parse")]
pub mod parse;
pub mod sequents;
#[cfg(feature = "serialize")]
mod serialize;

#[cfg(feature = "parse")]
pub use errors::ParseError;

pub use errors::Error;

/// Returns the sum of `left` and `right`.
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `add` sums two small numbers.
    #[test]
    fn two_plus_two_is_four() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
