// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::nets::ProofStructure;
use crate::occurrences::{Forest, OccId};
use crate::sequents::Sequent;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The serialized form of a proof structure: its sequent, whether Mix is
/// allowed, and its links as pairs of occurrence ids in the order they
/// were made.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Net {
    /// The sequent.
    sequent: Sequent,
    /// Whether Mix is allowed.
    mix: bool,
    /// The axiom links.
    links: Vec<(u32, u32)>,
}

impl Serialize for ProofStructure {
    /// Serializes the structure as its sequent, its Mix flag and its links.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Net {
            sequent: self.sequent().clone(),
            mix: self.mix(),
            links: self
                .links()
                .iter()
                .map(|&(x, y)| (x.get(), y.get()))
                .collect(),
        }
        .serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for ProofStructure {
    /// Deserializes a structure, rebuilding the forest of its sequent and
    /// validating the links as [`ProofStructure::from_links`] does; whether
    /// it is a proof net is for [`ProofStructure::is_correct`].
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let net = Net::deserialize(deserializer)?;
        let forest = Forest::try_from(net.sequent).map_err(serde::de::Error::custom)?;
        let links: Vec<(OccId, OccId)> = net
            .links
            .into_iter()
            .map(|(x, y)| (OccId::new(x), OccId::new(y)))
            .collect();
        ProofStructure::from_links(forest, net.mix, &links).map_err(serde::de::Error::custom)
    }
}
