// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The additive fast path: a sequent of two additive-only formulas,
//! `⊢ A, B` classically or `A ⊢ B` intuitionistically, is decided by a
//! recursion on pairs of subformula occurrences, memoized on the pair, in
//! time proportional to the product of the two sizes as long as the memo
//! holds every pair; it is emptied when it reaches the limit of the
//! options, which costs time and never an answer.
//!
//! Every rule of the additive fragment replaces one formula of a two-formula
//! sequent by a subformula, so every sequent of a proof is a pair of
//! occurrences, one below each root: a `⊤` closes, a `&` needs both
//! subformulas against the other side, a `⊕` one of them, two dual literals
//! are an axiom, and nothing else proves anything (`0` in particular). `&`
//! is invertible and decomposed first; which `⊕` to decompose is a real
//! choice (a `&` below the other formula's `⊕` may need both sides of this
//! one), so both are tried, and a pair reached along two orders is decided
//! once through the memo. The rules are the same two-sided: `&` on a
//! hypothesis is `⊕L` on the one-sided sequent and so on, every sequent
//! has one goal by itself, and neither weakening nor Mix can help a
//! sequent of two formulas, so the procedure is the same in every mode.

use super::focus::Search;
use super::memory::{Account, bytes_of};
use super::{Options, Reason, Refutation, Statistics, Verdict};
use crate::fragment::Mode;
use crate::hash::HashMap;
use crate::occurrences::{Forest, OccId};
use crate::proofs::{Node, NodeId, Proof, Side};
use crate::sequents::Kind;

/// Runs the additive fast path on the forest of a sequent of exactly two
/// additive-only formulas under `mode`, polling `stop` at every pair of
/// occurrences, and returns the verdict with the statistics of the run.
pub(crate) fn search(
    forest: &Forest,
    mode: Mode,
    options: &Options,
    stop: &mut dyn FnMut() -> bool,
) -> (Verdict, Statistics) {
    let account = Account::new(options.memory_limit);
    let (result, nodes, statistics) = search_goal(forest, forest.roots(), options, &account, stop);
    let verdict = match result {
        Ok(Some(root)) => {
            let proof = Proof::new(forest.clone(), nodes, root)
                .expect("the engine pushes premises before conclusions");
            debug_assert_eq!(proof.check(mode), Ok(()), "the engine's proof");
            Verdict::Proved(Box::new(proof))
        }
        Ok(None) => Verdict::Unprovable(Refutation::Exhausted),
        Err(reason) => Verdict::Unknown(reason),
    };
    (verdict, statistics)
}

/// Runs the additive fast path on a goal of exactly two additive-only
/// occurrences of the forest, the roots or any other pair, polling `stop`
/// at every pair of occurrences. Returns the node proving the goal (`None`
/// when it is unprovable, or the reason the search gave up), the arena the
/// node lives in, and the statistics. The mode plays no part: the additive
/// rules keep one goal by themselves, and neither weakening nor Mix can
/// help a sequent of two formulas. The memo and the arena are charged to
/// `account`.
pub(crate) fn search_goal(
    forest: &Forest,
    goal: &[OccId],
    options: &Options,
    account: &Account,
    stop: &mut dyn FnMut() -> bool,
) -> (Search, Vec<Node>, Statistics) {
    let [x, y] = goal else {
        unreachable!("the dispatch sends goals of two formulas here");
    };
    let mut engine = Engine {
        forest,
        memo: HashMap::default(),
        memo_limit: options.memo_limit,
        memo_peak: 0,
        nodes: Vec::new(),
        statistics: Statistics::default(),
        depth: 0,
        recursion_limit: options.recursion_limit,
        account,
        charged: 0,
        stop,
    };
    let result = engine
        .pair(*x, *y)
        .map_err(|r| super::focus::reason(r, options));
    let statistics = Statistics {
        memo_entries: engine.memo_peak,
        ..engine.statistics
    };
    (result, engine.nodes, statistics)
}

/// The state of one run: the problem, the memo of pairs, the proof arena
/// and the counters.
struct Engine<'a> {
    /// The problem.
    forest: &'a Forest,
    /// The pairs decided: the node proving the pair, or `None`.
    memo: HashMap<(OccId, OccId), Option<NodeId>>,
    /// The most pairs the memo holds; zero switches it off.
    memo_limit: usize,
    /// The most pairs the memo held at once.
    memo_peak: usize,
    /// The proof arena.
    nodes: Vec<Node>,
    /// The counters: `nodes` is the pairs visited, memo hits included.
    statistics: Statistics,
    /// The nesting of pairs right now.
    depth: u32,
    /// The deepest nesting allowed.
    recursion_limit: u32,
    /// Who is charged the memo and the arena.
    account: &'a Account,
    /// The bytes of the memo and the arena that were charged.
    charged: usize,
    /// The caller's stop condition.
    stop: &'a mut dyn FnMut() -> bool,
}

impl Engine<'_> {
    /// Decides the sequent of the two occurrences: the node proving it,
    /// `None` when it is unprovable, or the reason the search stops.
    fn pair(&mut self, x: OccId, y: OccId) -> Result<Option<NodeId>, Reason> {
        self.statistics.nodes += 1;
        if (self.stop)() {
            return Err(Reason::Stopped);
        }
        self.settle()?;
        if let Some(&node) = self.memo.get(&(x, y)) {
            self.statistics.memo_hits += 1;
            return Ok(node);
        }
        if self.depth >= self.recursion_limit {
            return Err(Reason::RecursionLimit);
        }
        self.depth += 1;
        let result = self.decide(x, y);
        self.depth -= 1;
        let node = result?;
        if self.memo_limit != 0 {
            // A full memo is emptied: every entry is a fact the search can
            // find again, and the arena keeps the nodes of the proofs.
            if self.memo.len() >= self.memo_limit {
                self.memo.clear();
            }
            self.memo.insert((x, y), node);
            self.memo_peak = self.memo_peak.max(self.memo.len());
        }
        Ok(node)
    }

    /// The rules on the pair: `⊤` first, then the invertible `&` on either
    /// side, then the axiom, then a `⊕` on either side, one side at a time.
    fn decide(&mut self, x: OccId, y: OccId) -> Result<Option<NodeId>, Reason> {
        let f = self.forest;
        let (kx, ky) = (f.kind(x), f.kind(y));
        if kx == Kind::Top {
            return Ok(Some(self.push(Node::Top(x))));
        }
        if ky == Kind::Top {
            return Ok(Some(self.push(Node::Top(y))));
        }
        // `&` on either side: both subformulas against the other formula.
        for (o, other, x_first) in [(x, y, true), (y, x, false)] {
            if f.kind(o) != Kind::With {
                continue;
            }
            let (l, r) = (f.left(o).unwrap(), f.right(o).unwrap());
            let Some(left) = self.ordered(l, other, x_first)? else {
                return Ok(None);
            };
            let Some(right) = self.ordered(r, other, x_first)? else {
                return Ok(None);
            };
            return Ok(Some(self.push(Node::With(o, left, right))));
        }
        // Two literals: an axiom if they are dual; `0` and a lone literal
        // prove nothing.
        if kx.is_literal() && ky.is_literal() {
            let dual = f.atom(x) == f.atom(y) && f.sign(x) != f.sign(y);
            return Ok(dual.then(|| self.push(Node::Ax(x, y))));
        }
        // `⊕` on either side: one subformula against the other formula.
        for (o, other, x_first) in [(x, y, true), (y, x, false)] {
            if f.kind(o) != Kind::Plus {
                continue;
            }
            for (side, sub) in [
                (Side::Left, f.left(o).unwrap()),
                (Side::Right, f.right(o).unwrap()),
            ] {
                if let Some(premise) = self.ordered(sub, other, x_first)? {
                    return Ok(Some(self.push(Node::Plus(o, side, premise))));
                }
            }
        }
        Ok(None)
    }

    /// Decides the pair of a subformula `sub` of one side and the other
    /// formula, keeping the memo's order: the occurrence below the first
    /// root first.
    fn ordered(
        &mut self,
        sub: OccId,
        other: OccId,
        sub_first: bool,
    ) -> Result<Option<NodeId>, Reason> {
        if sub_first {
            self.pair(sub, other)
        } else {
            self.pair(other, sub)
        }
    }

    /// Charges the account what the memo and the arena grew by, and makes
    /// room when that passes the bound: the memo goes first. Fails when
    /// the arena alone is over the bound, or holds as many nodes as an id
    /// can name; a pair adds one node at most.
    fn settle(&mut self) -> Result<(), Reason> {
        /// The bytes of a slot of the memo: a pair, a node, a control byte.
        const SLOT: usize = size_of::<((OccId, OccId), Option<NodeId>)>() + 1;
        if self.nodes.len() >= u32::MAX as usize {
            return Err(Reason::IndexLimit);
        }
        let held = self.memo.capacity() * SLOT + bytes_of(&self.nodes);
        if held == self.charged {
            return Ok(());
        }
        self.account.resize(self.charged, held);
        self.charged = held;
        if self.account.over() {
            self.memo = HashMap::default();
            self.account.resize(self.charged, bytes_of(&self.nodes));
            self.charged = bytes_of(&self.nodes);
            if self.account.over() {
                return Err(Reason::MemoryLimit(self.account.limit()));
            }
        }
        Ok(())
    }

    /// Appends a node to the arena and returns its id.
    fn push(&mut self, node: Node) -> NodeId {
        let id = NodeId::new(self.nodes.len() as u32);
        self.nodes.push(node);
        id
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::Sequent;
    use crate::search::generate::Rng;
    use crate::search::{Engine as Which, prove};

    /// Parses `input`.
    fn sequent(input: &str) -> Sequent {
        input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"))
    }

    /// Decides `input` under `mode` with the engine given, checks the proof
    /// if there is one, and returns whether it is provable.
    fn decide(input: &str, mode: Mode, engine: Which) -> bool {
        let s = sequent(input);
        let outcome = prove(&s, mode, &Options::default().engine(Some(engine)))
            .unwrap_or_else(|e| panic!("{input:?} by {engine}: {e}"));
        assert_eq!(outcome.engine, engine);
        match outcome.verdict {
            Verdict::Proved(proof) => {
                assert_eq!(proof.sequent(), &s);
                proof.check(mode).unwrap_or_else(|e| {
                    panic!("{input:?} by {engine}: {}", e.describe(proof.forest()))
                });
                true
            }
            Verdict::Unprovable(_) => false,
            Verdict::Unknown(reason) => panic!("{input:?} by {engine}: {reason}"),
        }
    }

    /// A memo that fills up is emptied and the verdict stays: the identity
    /// of depth 8, which decides thousands of pairs, with room for 64.
    #[test]
    fn capped_memo() {
        let instance = crate::families::find("additive").unwrap().instance(8, 0);
        let options = Options::default().memo_limit(64);
        let outcome = prove(&instance.sequent, instance.mode, &options).unwrap();
        assert_eq!(outcome.engine, Which::Additive);
        assert_eq!(outcome.statistics.memo_entries, 64);
        let Verdict::Proved(proof) = outcome.verdict else {
            panic!("the identity is provable");
        };
        assert_eq!(proof.check(instance.mode), Ok(()));
    }

    /// The additive fragment's textbook sequents, classically and
    /// intuitionistically.
    #[test]
    fn additive_sequents() {
        for (input, expected) in [
            ("a |- a", true),
            ("a |- b", false),
            ("a & b |- a", true),
            ("a |- a & b", false),
            ("a |- a + b", true),
            ("a + b |- a", false),
            ("a & b |- a + b", true),
            ("a + b |- a & b", false),
            ("a & b |- b & a", true),
            ("a + b |- b + a", true),
            ("a & (b + c) |- (a & b) + (a & c)", false),
            ("(a & b) + (a & c) |- a & (b + c)", true),
            ("|- top, 0", true),
            ("0 |- a", true),
            ("a |- 0", false),
            ("top |- a", false),
            ("a |- top", true),
            ("0 |- top", true),
            ("a & top |- a", true),
            ("a |- a & top", true),
            ("a + 0 |- a", true),
            ("a |- a + 0", true),
            ("0 + b |- a", false),
            ("(a + top) & b |- b", true),
            ("|- a & b, ~a + ~b", true),
            ("|- a & b, ~a", false),
            // A `&` below the second formula's `⊕` needs both sides of the
            // first formula's `⊕`, so the second `⊕` must be tried too.
            ("|- ~c + ~a, b + (c & a)", true),
            ("b + (c & a) |- b + (c & a)", true),
            ("c & a |- b + (c & a)", true),
        ] {
            for mode in [
                Mode::CLASSICAL,
                Mode::CLASSICAL.affine(),
                Mode::CLASSICAL.with_mix(),
            ] {
                assert_eq!(
                    decide(input, mode, Which::Additive),
                    expected,
                    "{input:?} {mode}"
                );
                assert_eq!(
                    decide(input, mode, Which::Focus),
                    expected,
                    "{input:?} {mode}"
                );
            }
            if !input.starts_with("|-") {
                let i = Mode::INTUITIONISTIC;
                assert_eq!(decide(input, i, Which::Additive), expected, "{input:?} {i}");
                assert_eq!(decide(input, i, Which::TwoSided), expected, "{input:?} {i}");
            }
        }
    }

    /// A random additive formula in the parser's syntax.
    fn formula(rng: &mut Rng, depth: usize) -> String {
        if depth == 0 || rng.one_in(4) {
            return match rng.below(8) {
                0 => "top".into(),
                1 => "0".into(),
                n => ["a", "b", "c", "~a", "~b", "~c"][n - 2].into(),
            };
        }
        let (l, r) = (formula(rng, depth - 1), formula(rng, depth - 1));
        if rng.one_in(2) {
            format!("({l} & {r})")
        } else {
            format!("({l} + {r})")
        }
    }

    /// On random pairs of additive formulas the fast path agrees with the
    /// focused engine, classically and, where the pair is an intuitionistic
    /// sequent, with the two-sided engine.
    #[test]
    fn agrees_with_the_focused_engine() {
        let mut rng = Rng::new(11);
        let (mut compared, mut provable) = (0, 0);
        for _ in 0..300 {
            let input = format!("|- {}, {}", formula(&mut rng, 3), formula(&mut rng, 3));
            let expected = decide(&input, Mode::CLASSICAL, Which::Focus);
            assert_eq!(
                decide(&input, Mode::CLASSICAL, Which::Additive),
                expected,
                "{input:?}"
            );
            let i = Mode::INTUITIONISTIC;
            if prove(&sequent(&input), i, &Options::default()).is_ok() {
                let expected = decide(&input, i, Which::TwoSided);
                assert_eq!(decide(&input, i, Which::Additive), expected, "{input:?}");
            }
            compared += 1;
            provable += usize::from(expected);
        }
        assert!(
            provable > 30 && provable < compared,
            "{provable} of {compared}"
        );
    }

    /// The memo bounds the work by the product of the sizes: two deep `⊕`
    /// towers over atoms that never match are refuted with fewer pairs
    /// than there are subformula pairs, not one per order of choices.
    #[test]
    fn memoized() {
        let mut left = String::from("a");
        let mut right = String::from("~b");
        for _ in 0..7 {
            left = format!("({left} + {left})");
            right = format!("({right} + {right})");
        }
        let s = sequent(&format!("|- {left}, {right}"));
        let outcome = prove(&s, Mode::CLASSICAL, &Options::default()).unwrap();
        assert_eq!(outcome.engine, Which::Additive);
        assert!(matches!(outcome.verdict, Verdict::Unprovable(_)));
        // 2⁸ − 1 subformulas on each side, so at most that squared pairs
        // decided; every other visit is a memo hit.
        let s = outcome.statistics;
        assert!(s.memo_entries <= 255 * 255, "{}", s.memo_entries);
        assert!(s.memo_hits > 0);
        assert_eq!(s.memo_entries as u64 + s.memo_hits, s.nodes);
    }
}
