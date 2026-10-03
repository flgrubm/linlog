// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::proofs::Proof;
use crate::fragment::{Fragment, Mode};
use crate::search::{Engine, Outcome as Out, Reason, Refutation, Statistics, Verdict};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The fragments a name can stand for, in the order of their names.
const NAMED: [Fragment; 6] = [
    Fragment::MLL,
    Fragment::MLL_WITH_UNITS,
    Fragment::ALL,
    Fragment::MALL,
    Fragment::MELL,
    Fragment::LL,
];

impl Serialize for Fragment {
    /// Serializes the fragment as its name, such as `"MALL"`.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

impl<'a> Deserialize<'a> for Fragment {
    /// Deserializes a fragment from its name, giving the named fragment,
    /// which contains every fragment of that name.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        // The intuitionistic names put an `I` in front: `ILL`, `IMLL`, …
        let classical = name.strip_prefix('I').unwrap_or(&name);
        NAMED
            .into_iter()
            .find(|f| f.name() == classical)
            .ok_or_else(|| {
                serde::de::Error::custom(format!(
                    "unknown fragment {name:?}, expected one of {}",
                    NAMED.map(|f| format!("{:?}", f.name())).join(", ")
                ))
            })
    }
}

/// The serialized form of a mode: its three flags by name.
#[derive(Serialize, Deserialize)]
#[serde(remote = "Mode")]
struct ModeDef {
    /// Intuitionistic rather than classical.
    intuitionistic: bool,
    /// Weakening allowed.
    affine: bool,
    /// Mix allowed.
    mix: bool,
}

impl Serialize for Mode {
    /// Serializes the mode as an object of its three flags.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ModeDef::serialize(self, serializer)
    }
}

impl<'a> Deserialize<'a> for Mode {
    /// Deserializes a mode from an object of its three flags.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        ModeDef::deserialize(deserializer)
    }
}

/// The serialized form of a reason: a tag, with the bound for the copy
/// bound and for the memory limit.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum Why {
    /// The stop condition fired.
    Stopped,
    /// The recursion limit was reached.
    RecursionLimit,
    /// Every level up to this copy bound hit it.
    CopyBound(u32),
    /// The search held this many bytes with its memo emptied.
    MemoryLimit(u64),
    /// A structure of the search outgrew its indices.
    IndexLimit,
}

impl From<Reason> for Why {
    /// Converts a reason into its serialized form.
    fn from(r: Reason) -> Self {
        match r {
            Reason::Stopped => Why::Stopped,
            Reason::RecursionLimit => Why::RecursionLimit,
            Reason::CopyBound(n) => Why::CopyBound(n),
            Reason::MemoryLimit(bytes) => Why::MemoryLimit(bytes),
            Reason::IndexLimit => Why::IndexLimit,
        }
    }
}

/// The serialized form of a refutation: a tag, with the counts that rule
/// a proof out.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum WhyNot<'a> {
    /// The search was exhaustive.
    Exhausted,
    /// An atom's literals cannot pair up.
    Unbalanced {
        /// The atom's name.
        atom: &'a str,
        /// The least excess of positive over negative literals.
        least: i32,
        /// The greatest.
        most: i32,
    },
    /// The count equation fails.
    Equation {
        /// The formulas.
        formulas: u64,
        /// The formulas the equation asks for.
        needed: i128,
        /// The `⊗`.
        tensors: u64,
        /// The `⅋`.
        pars: u64,
        /// The `1`.
        ones: u64,
        /// The `⊥`.
        bottoms: u64,
        /// Whether Mix was allowed.
        mix: bool,
    },
}

impl<'a> From<&'a Refutation> for WhyNot<'a> {
    /// Converts a refutation into its serialized form.
    fn from(r: &'a Refutation) -> Self {
        match r {
            Refutation::Unbalanced {
                name, least, most, ..
            } => WhyNot::Unbalanced {
                atom: name,
                least: *least,
                most: *most,
            },
            &Refutation::Equation {
                formulas,
                tensors,
                pars,
                ones,
                bottoms,
                mix,
            } => WhyNot::Equation {
                formulas,
                needed: r.needed().unwrap_or_default(),
                tensors,
                pars,
                ones,
                bottoms,
                mix,
            },
            _ => WhyNot::Exhausted,
        }
    }
}

/// The serialized form of the statistics: its counters by name.
#[derive(Serialize)]
#[serde(remote = "Statistics")]
struct StatisticsDef {
    /// Stable sequents visited.
    nodes: u64,
    /// Visits answered from the memo.
    memo_hits: u64,
    /// The peak size of the memo.
    memo_entries: usize,
    /// Context splits examined.
    splits: u64,
    /// Axiom links the net engine tried.
    links: u64,
    /// Exact acyclicity tests the net engine ran.
    tests: u64,
    /// The largest copy bound the deepening reached.
    copies: u32,
}

/// The serialized form of an outcome: the verdict as a word, the reason for
/// `unknown`, how the search ran, and for `proved` the proof's sequent and
/// nodes as keys of the outcome itself, so that the outcome reads as a
/// proof file too.
#[derive(Serialize)]
struct Outcome<'a> {
    /// `proved`, `unprovable` or `unknown`.
    verdict: &'static str,
    /// Why the search could not decide.
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<Why>,
    /// Why the sequent is unprovable.
    #[serde(skip_serializing_if = "Option::is_none")]
    refutation: Option<WhyNot<'a>>,
    /// The fragment searched in, by its name in the mode.
    fragment: &'static str,
    /// The mode searched in.
    mode: Mode,
    /// The engine that ran.
    engine: Engine,
    /// What the search cost.
    #[serde(with = "StatisticsDef")]
    statistics: Statistics,
    /// The proof, for `proved`.
    #[serde(flatten)]
    proof: Option<Proof>,
}

impl Serialize for Out {
    /// Serializes the outcome as its verdict, the fragment, mode and engine
    /// of the search, the statistics, and the proof if there is one.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let (verdict, reason, refutation, proof) = match &self.verdict {
            Verdict::Proved(p) => ("proved", None, None, Some(Proof::from(&**p))),
            Verdict::Unprovable(r) => ("unprovable", None, Some(WhyNot::from(r)), None),
            Verdict::Unknown(r) => ("unknown", Some(Why::from(*r)), None, None),
        };
        Outcome {
            verdict,
            reason,
            refutation,
            fragment: self.fragment.name_in(self.mode),
            mode: self.mode,
            engine: self.engine,
            statistics: self.statistics,
            proof,
        }
        .serialize(serializer)
    }
}

impl Serialize for Engine {
    /// Serializes the engine as its name, such as `"focus"`.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
