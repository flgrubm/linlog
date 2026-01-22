// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::modes::{Efficient, Rich};
use super::symbols::{LL, Symbol};
use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(bound = "Box<[Symbol<LL, Rich>]>: Serialize + for<'a> Deserialize<'a>")]
pub struct RichSequent {
    lhs_symbols: Box<[Symbol<LL, Rich>]>,
    lhs_terms: Box<[usize]>,
    rhs_symbols: Box<[Symbol<LL, Rich>]>,
    rhs_terms: Box<[usize]>,
    variable_names: Box<[String]>,
}

// TODO: figure out how to implement the Serialize trait for Symbol
// #[derive(Debug, Serialize, Deserialize)]
// enum Abb {
//     Variant(std::convert::Infallible),
//     Other,
// }
