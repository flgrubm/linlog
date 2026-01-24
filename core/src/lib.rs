// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

// Source - https://stackoverflow.com/a
// Posted by M. Hamza Rajput, modified by community. See post 'Timeline' for change history
// Retrieved 2026-01-22, License - CC BY-SA 4.0

#![allow(dead_code)]
#![allow(unused_variables)]

mod errors;
pub mod sequents;
pub(crate) mod utils;

pub use errors::Error;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
