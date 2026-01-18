// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

mod errors;
pub mod sequents;

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
