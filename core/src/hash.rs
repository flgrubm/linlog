// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The hash tables of this crate. Their keys are small and many (terms while
//! hash-consing, occurrence sets in the memo tables of proof search), so they
//! use foldhash, which hashes a few words in a few cycles, instead of the
//! standard library's SipHash. The seed is fixed: nothing here faces
//! untrusted input, and a run that hashes the same way every time is
//! reproducible, on wasm as well, where foldhash has no clock to seed from.

/// The hasher every table in this crate uses.
pub(crate) type BuildHasher = foldhash::fast::FixedState;

/// A hash map keyed with the crate's hasher.
pub(crate) type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasher>;

/// A hash set keyed with the crate's hasher.
pub(crate) type HashSet<T> = std::collections::HashSet<T, BuildHasher>;
