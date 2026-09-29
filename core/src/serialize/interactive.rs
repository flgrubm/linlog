// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::fragment::Mode;
use crate::occurrences::{Forest, OccId};
use crate::proofs::interactive::Interactive as State;
use crate::proofs::{InfId, Inference, Rule};
use crate::sequents::Sequent;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The serialized form of one inference: its sequent as occurrence ids,
/// and for a closed one the rule's name, the principal position and the
/// premises' indices; an open goal is its sequent alone.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Step {
    /// The sequent, as occurrence ids in ascending order.
    sequent: Vec<u32>,
    /// The rule, absent for an open goal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rule: Option<Rule>,
    /// The position of the principal formula, absent for an axiom, Mix or
    /// an open goal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    principal: Option<usize>,
    /// The premises, absent for a leaf.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    premises: Vec<u32>,
}

/// The serialized form of a proof in progress: the sequent, the mode, the
/// inferences with the root first, and the goals the steps closed, in
/// order.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Interactive {
    /// The sequent being proved.
    sequent: Sequent,
    /// The mode.
    mode: Mode,
    /// The inferences.
    inferences: Vec<Step>,
    /// The steps, each the inference it closed.
    history: Vec<u32>,
}

impl From<&State> for Interactive {
    /// Converts a state into its serialized form.
    fn from(state: &State) -> Self {
        Self {
            sequent: state.sequent().clone(),
            mode: state.mode(),
            inferences: state
                .inferences()
                .iter()
                .map(|inference| Step {
                    sequent: inference.sequent.iter().map(|o| o.get()).collect(),
                    rule: (inference.rule != Rule::Open).then_some(inference.rule),
                    principal: inference.principal,
                    premises: inference.premises.iter().map(|p| p.get()).collect(),
                })
                .collect(),
            history: state.history().iter().map(|id| id.get()).collect(),
        }
    }
}

impl TryFrom<Interactive> for State {
    type Error = crate::Error;

    /// Rebuilds the forest and the state over it, checking that the
    /// inferences fit together and that every closed one is what its rule
    /// yields.
    fn try_from(proxy: Interactive) -> Result<State, Self::Error> {
        let forest = Forest::try_from(proxy.sequent)?;
        let inferences = proxy
            .inferences
            .into_iter()
            .map(|step| Inference {
                sequent: step.sequent.into_iter().map(OccId::new).collect(),
                rule: step.rule.unwrap_or(Rule::Open),
                principal: step.principal,
                premises: step.premises.into_iter().map(InfId::new).collect(),
            })
            .collect();
        let history = proxy.history.into_iter().map(InfId::new).collect();
        State::from_parts(forest, proxy.mode, inferences, history)
    }
}

impl Serialize for State {
    /// Serializes the state as its sequent, mode, inferences and steps.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Interactive::from(self).serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for State {
    /// Deserializes a state and checks that it is consistent; see
    /// [`Interactive`](State).
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let proxy = Interactive::deserialize(deserializer)?;
        State::try_from(proxy).map_err(serde::de::Error::custom)
    }
}
