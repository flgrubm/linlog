// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Export of sequents: to LaTeX ([`latex`](crate::export::latex), feature `latex`) and to Typst
//! ([`typst`](crate::export::typst), feature `typst`). Every function is a pure function of its
//! input returning the text, as a fragment to paste or as a standalone
//! document ([`Form`](crate::export::Form)); the output is deterministic. Both targets write
//! formulas with the bracketing of `Display`: every binary subformula of a
//! formula in brackets.

#[cfg(feature = "latex")]
pub mod latex;
/// The symbol tables and the printers the targets share.
#[cfg(any(feature = "latex", feature = "typst"))]
mod notation;
#[cfg(feature = "typst")]
pub mod typst;

/// Whether an export is a fragment to paste into a document or a document
/// of its own.
#[cfg(any(feature = "latex", feature = "typst"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Form {
    /// The sequent alone, for a document that loads the
    /// packages it needs.
    #[default]
    Fragment,
    /// A complete document that compiles on its own, cropped to its
    /// content.
    Standalone,
}
