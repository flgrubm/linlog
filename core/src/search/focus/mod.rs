// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The focused sequent engine: backward search over occurrence bitsets with
//! a memo of stable sequents, as the `MALL-Seq` specification states it.

/// The count invariants the engine prunes with.
mod counts;
