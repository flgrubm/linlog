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
