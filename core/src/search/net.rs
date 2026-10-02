// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Proof-net search for unit-free MLL, with or without Mix. A cut-free
//! proof of MLL is determined by its axiom linking, so the search
//! enumerates linkings over a [`ProofStructure`] by backtracking and keeps
//! the first one the correctness criterion accepts.
//!
//! Before any link, the count equation `c = t − p + 2` (`c` conclusions,
//! `t` tensor and `p` par occurrences; `≥` with Mix) and the balance of
//! every atom reject most unprovable sequents at once. Then each step
//! chooses the unlinked literal with the fewest admissible partners and
//! tries them in order. A partner is admissible when it is unlinked and
//! dual, when the two literals do not hang under a common `⊗` of one
//! conclusion, when the `⅋`-free skeleton does not join them already, and
//! when the partners of equal literal conclusions stay in the order of the
//! conclusions, which breaks that symmetry. The exact acyclicity test runs
//! after every link on a small structure and after every fourth link on a
//! large one, and on every complete linking, which is then a proof net: the
//! count equation is the connectedness equation of a complete acyclic
//! linking. The net is sequentialized into the proof term returned.

use super::{Options, Reason, Statistics, Stop, Verdict};
use crate::fragment::Mode;
use crate::nets::{ProofStructure, Scratch};
use crate::occurrences::{Forest, OccId, Sign};
use crate::sequents::{Atom, Kind};

/// The most occurrences a structure may have for the exact test to run
/// after every link by default.
const SMALL: usize = 200;

/// How many links go between two exact tests on a larger structure by
/// default.
const PERIOD: u32 = 4;

/// The raw index that stands for "no occurrence".
const NONE: u32 = u32::MAX;

/// Runs the proof-net engine on the forest of a sequent of unit-free MLL
/// under `mode`, polling `stop` once per literal chosen, and returns the
/// verdict with the statistics of the run and, for a proved sequent, the
/// proof net the proof was read off.
pub(crate) fn search(
    forest: &Forest,
    mode: Mode,
    options: &Options,
    stop: &mut dyn FnMut() -> bool,
) -> (Verdict, Statistics, Option<ProofStructure>) {
    if !counts_admit(forest, mode.mix) {
        return (Verdict::Unprovable, Statistics::default(), None);
    }
    let mut engine = Engine::new(forest, mode, options, Stop::Closure(stop));
    match engine.run() {
        Ok(true) => {
            let proof = engine
                .net
                .sequentialize()
                .expect("a complete linking that passed the exact test is a proof net");
            debug_assert_eq!(proof.check(mode), Ok(()), "the engine's proof");
            (
                Verdict::Proved(Box::new(proof)),
                engine.statistics,
                Some(engine.net),
            )
        }
        Ok(false) => (Verdict::Unprovable, engine.statistics, None),
        Err(reason) => (Verdict::Unknown(reason), engine.statistics, None),
    }
}

/// Returns whether the counts admit a proof: `c = t − p + 2` for `c`
/// conclusions, `t` tensor and `p` par occurrences (`c ≥ t − p + 2` with
/// Mix), and as many `a` as `~a` for every atom. Both are necessary for a
/// proof net to exist, and the first is the connectedness equation of a
/// complete acyclic linking.
fn counts_admit(forest: &Forest, mix: bool) -> bool {
    let (mut tensors, mut pars) = (0i64, 0i64);
    for o in forest.ids() {
        match forest.kind(o) {
            Kind::Tensor => tensors += 1,
            Kind::Par => pars += 1,
            _ => {}
        }
    }
    let conclusions = forest.roots().len() as i64;
    let needed = tensors - pars + 2;
    if if mix {
        conclusions < needed
    } else {
        conclusions != needed
    } {
        return false;
    }
    atoms(forest)
        .all(|a| forest.literals(a, Sign::Var).len() == forest.literals(a, Sign::DualVar).len())
}

/// Returns every atom of the forest's sequent.
fn atoms(forest: &Forest) -> impl Iterator<Item = Atom> {
    (0..forest.sequent().atom_names().len()).map(|a| Atom::new(a as u32))
}

/// A decision of the search: the literal chosen, and how far along the
/// literals of the other sign of its atom the search for a partner is.
#[derive(Clone, Copy, Debug)]
struct Frame {
    /// The literal being linked.
    literal: OccId,
    /// The position in the list of literals of the other sign at which the
    /// search for the next partner resumes.
    next: u32,
}

/// What the choice of the next literal found.
#[derive(Clone, Copy, Debug)]
enum Choice {
    /// Every literal is linked.
    Complete,
    /// Some unlinked literal has no admissible partner.
    DeadEnd,
    /// The unlinked literal with the fewest admissible partners.
    Literal(OccId),
}

/// The state of one run: the structure being linked, the working memory
/// of the exact test, the counts the choice of the next literal reads, the
/// stack of decisions and the counters. Nothing allocates once the run has
/// started.
struct Engine<'a> {
    /// The proof structure being linked.
    net: ProofStructure,
    /// Working memory for the exact test.
    scratch: Scratch,
    /// Per atom, the pairs of its literals still to link: as many `a` as
    /// `~a` are unlinked at every moment.
    remaining: Box<[u32]>,
    /// Per literal that is a conclusion, the previous conclusion that is
    /// the same literal, or `NONE`.
    copy_before: Box<[u32]>,
    /// Per literal that is a conclusion, the next conclusion that is the
    /// same literal, or `NONE`.
    copy_after: Box<[u32]>,
    /// The decisions made so far. Every frame but the top has its current
    /// link made; the top frame is looking for one.
    stack: Vec<Frame>,
    /// How many links go between two exact tests.
    period: u32,
    /// The counters.
    statistics: Statistics,
    /// The stop condition.
    stop: Stop<'a>,
}

impl<'a> Engine<'a> {
    /// Prepares a run on the forest, which must be one of unit-free MLL.
    fn new(forest: &Forest, mode: Mode, options: &Options, stop: Stop<'a>) -> Self {
        let net = ProofStructure::new(forest.clone(), mode.mix)
            .expect("the dispatch routes unit-free MLL only");
        let scratch = net.scratch();
        let remaining = atoms(forest)
            .map(|a| forest.literals(a, Sign::Var).len() as u32)
            .collect();
        // Equal literal conclusions, chained in id order.
        let n = forest.len();
        let mut copy_before = vec![NONE; n];
        let mut copy_after = vec![NONE; n];
        for a in atoms(forest) {
            for sign in [Sign::Var, Sign::DualVar] {
                let mut previous = NONE;
                for &l in forest.literals(a, sign) {
                    if forest.parent(l).is_some() {
                        continue;
                    }
                    if previous != NONE {
                        copy_after[previous as usize] = l.get();
                        copy_before[l.index()] = previous;
                    }
                    previous = l.get();
                }
            }
        }
        let period = match options.test_period {
            Some(period) => period.max(1),
            None if n <= SMALL => 1,
            None => PERIOD,
        };
        Self {
            scratch,
            remaining,
            copy_before: copy_before.into_boxed_slice(),
            copy_after: copy_after.into_boxed_slice(),
            stack: Vec::with_capacity(forest.all_literals().len() / 2 + 1),
            period,
            statistics: Statistics::default(),
            stop,
            net,
        }
    }

    /// Runs the search and returns whether a proof net was found, or the
    /// reason the search stopped. With links seeded, the search is the
    /// branch below them and ends when it backtracks up to them.
    fn run(&mut self) -> Result<bool, Reason> {
        self.explore(None, &mut Vec::new())
    }

    /// The search, and the enumeration of its cubes: with a `limit`, a
    /// branch that reaches that many links is recorded in `cubes` as the
    /// links made, and taken back instead of followed, so that the cubes
    /// are the branches of the first `limit` links that the tests do not
    /// reject; a proof net found within the limit ends the search as
    /// usual. Without a limit this is the whole search.
    fn explore(
        &mut self,
        limit: Option<usize>,
        cubes: &mut Vec<Vec<(OccId, OccId)>>,
    ) -> Result<bool, Reason> {
        // A dead end before any link is a refutation; a complete structure
        // without a link would have no literal, which the counts exclude.
        let Choice::Literal(first) = self.decide()? else {
            return Ok(false);
        };
        self.stack.push(Frame {
            literal: first,
            next: 0,
        });
        loop {
            let Some(y) = self.next_partner() else {
                self.stack.pop();
                if self.stack.is_empty() {
                    return Ok(false);
                }
                self.unlink();
                continue;
            };
            let x = self.stack.last().unwrap().literal;
            self.link(x, y);
            let complete = self.net.is_complete();
            if complete || (self.net.links().len() as u32).is_multiple_of(self.period) {
                self.statistics.tests += 1;
                if !self.net.is_acyclic(&mut self.scratch) {
                    self.unlink();
                    continue;
                }
            }
            if complete {
                return Ok(true);
            }
            if limit == Some(self.net.links().len()) {
                cubes.push(self.net.links().to_vec());
                self.unlink();
                continue;
            }
            match self.decide()? {
                Choice::Literal(literal) => self.stack.push(Frame { literal, next: 0 }),
                Choice::DeadEnd => self.unlink(),
                Choice::Complete => unreachable!("an incomplete structure has an unlinked literal"),
            }
        }
    }

    /// Polls the stop condition, counts a node, and chooses the next
    /// literal.
    fn decide(&mut self) -> Result<Choice, Reason> {
        self.statistics.nodes += 1;
        if self.stop.fired(1) {
            return Err(Reason::Stopped);
        }
        Ok(self.choose())
    }

    /// Makes the links of a cube, on an engine without links, so that
    /// `run` searches the branch below them.
    fn seed(&mut self, links: &[(OccId, OccId)]) {
        debug_assert!(
            self.net.links().is_empty(),
            "a seed goes on an empty structure"
        );
        for &(x, y) in links {
            self.link(x, y);
        }
    }

    /// Takes every link back and forgets the decisions, keeping the
    /// counters, so that the engine can run another cube.
    fn reset(&mut self) {
        self.stack.clear();
        while !self.net.links().is_empty() {
            self.unlink();
        }
    }

    /// Chooses the unlinked literal with the fewest admissible partners,
    /// the first in atom order, `a` before `~a`, then id order among
    /// equals; or reports that some literal has none, or that every
    /// literal is linked. Every unlinked literal is looked at, so a dead
    /// end anywhere is found now rather than after more links.
    fn choose(&self) -> Choice {
        let forest = self.net.forest();
        let mut best: Option<(usize, OccId)> = None;
        for (a, &remaining) in self.remaining.iter().enumerate() {
            if remaining == 0 {
                continue;
            }
            let atom = Atom::new(a as u32);
            for sign in [Sign::Var, Sign::DualVar] {
                let partners = forest.literals(atom, !sign);
                for &x in forest.literals(atom, sign) {
                    if self.net.partner(x).is_some() {
                        continue;
                    }
                    // Counting stops once the literal cannot beat the best.
                    let bound = best.map_or(usize::MAX, |(count, _)| count);
                    let mut count = 0;
                    for &y in partners {
                        if self.net.partner(y).is_none() && self.admissible(x, y) {
                            count += 1;
                            if count >= bound {
                                break;
                            }
                        }
                    }
                    if count == 0 {
                        return Choice::DeadEnd;
                    }
                    if count < bound {
                        best = Some((count, x));
                    }
                }
            }
        }
        match best {
            Some((_, literal)) => Choice::Literal(literal),
            None => Choice::Complete,
        }
    }

    /// Finds the next admissible partner of the top frame's literal, moves
    /// the frame past it and returns it, or `None` when none is left.
    fn next_partner(&mut self) -> Option<OccId> {
        let Frame { literal: x, next } = *self.stack.last().unwrap();
        let forest = self.net.forest();
        let partners = forest.literals(forest.atom(x).unwrap(), !forest.sign(x).unwrap());
        for (i, &y) in partners.iter().enumerate().skip(next as usize) {
            if self.net.partner(y).is_none() && self.admissible(x, y) {
                self.stack.last_mut().unwrap().next = i as u32 + 1;
                return Some(y);
            }
        }
        None
    }

    /// Returns whether linking two unlinked dual literals passes the
    /// constant-time rejections: they do not hang under a common `⊗` of
    /// one conclusion (the tree path and the link would be a cycle that
    /// the switchings keeping the path's `⅋` premises keep, and it stays
    /// one whatever is linked later), the `⅋`-free skeleton does not join
    /// them already (the path and the link would be a cycle every
    /// switching keeps), and the partners of equal literal conclusions
    /// stay in the order of the conclusions.
    fn admissible(&self, x: OccId, y: OccId) -> bool {
        let forest = self.net.forest();
        if let Some(a) = forest.lca(x, y)
            && forest.kind(a) == Kind::Tensor
        {
            return false;
        }
        if self.net.same_component(x, y) {
            return false;
        }
        self.ordered(x, y) && self.ordered(y, x)
    }

    /// Returns whether linking `x` to `y` keeps the partners of the
    /// conclusions that are the same literal as `x` in the order of those
    /// conclusions. Swapping two such conclusions maps proof nets to proof
    /// nets, so among the linkings that differ only by such swaps the one
    /// with ascending partners is enough to look for.
    fn ordered(&self, x: OccId, y: OccId) -> bool {
        let before = self.copy_before[x.index()];
        if before != NONE
            && let Some(p) = self.net.partner(OccId::new(before))
            && p > y
        {
            return false;
        }
        let after = self.copy_after[x.index()];
        if after != NONE
            && let Some(q) = self.net.partner(OccId::new(after))
            && q < y
        {
            return false;
        }
        true
    }

    /// Makes a link and keeps the counts current.
    fn link(&mut self, x: OccId, y: OccId) {
        self.net.link(x, y);
        let atom = self.net.forest().atom(x).unwrap();
        self.remaining[atom.index()] -= 1;
        self.statistics.links += 1;
    }

    /// Takes back the last link and keeps the counts current.
    fn unlink(&mut self) {
        let (x, _) = self.net.unlink().expect("a link to take back");
        let atom = self.net.forest().atom(x).unwrap();
        self.remaining[atom.index()] += 1;
    }
}

#[cfg(all(test, feature = "parse"))]
pub(super) mod tests {
    use super::*;
    use crate::search::generate::{self, Rng, Rules};
    use crate::search::{Engine, focus};
    use crate::sequents::Sequent;
    use std::time::{Duration, Instant};

    /// Runs the net engine on `input` under `mode` with `options`, checks
    /// the proof if there is one against the checker and against the net,
    /// and returns the verdict and the statistics.
    fn run(input: &str, mode: Mode, options: &Options) -> (Verdict, Statistics) {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let forest = Forest::new(&s).unwrap();
        let (verdict, statistics, net) = search(&forest, mode, options, &mut || false);
        match (&verdict, &net) {
            (Verdict::Proved(proof), Some(net)) => {
                assert_eq!(proof.sequent(), &s);
                proof
                    .check(mode)
                    .unwrap_or_else(|e| panic!("{input:?}: the proof is wrong: {e}"));
                assert_eq!(net.is_correct(), Ok(()), "{input:?}");
                let again = ProofStructure::from_proof(proof, mode.mix).unwrap();
                assert_eq!(sorted(&again), sorted(net), "{input:?}: the proof's net");
            }
            (Verdict::Proved(_), None) | (_, Some(_)) => {
                panic!("{input:?}: a net exactly when proved")
            }
            _ => {}
        }
        (verdict, statistics)
    }

    /// The links of a structure as sorted pairs.
    fn sorted(net: &ProofStructure) -> Vec<(OccId, OccId)> {
        let mut links: Vec<(OccId, OccId)> = net
            .links()
            .iter()
            .map(|&(x, y)| (x.min(y), x.max(y)))
            .collect();
        links.sort();
        links
    }

    /// Whether `input` is provable under `mode`, panicking on `Unknown`.
    fn provable(input: &str, mode: Mode) -> bool {
        match run(input, mode, &Options::default()).0 {
            Verdict::Proved(_) => true,
            Verdict::Unprovable => false,
            Verdict::Unknown(reason) => panic!("{input:?}: {reason}"),
        }
    }

    /// The mode a rule set of the generator stands for.
    fn mode_for(mix: bool) -> Mode {
        if mix {
            Mode::CLASSICAL.with_mix()
        } else {
            Mode::CLASSICAL
        }
    }

    /// The classic small sequents get the verdicts the textbooks give
    /// them, with and without Mix.
    #[test]
    fn classic_sequents() {
        let m = Mode::CLASSICAL;
        for (input, expected) in [
            ("|- ~a, a", true),
            ("a |- a", true),
            ("|- a, a", false),
            ("|-", false),
            ("|- a, ~a, a, ~a", false),
            ("a * b |- a * b", true),
            ("|- a * b, ~a par ~b", true),
            ("|- a * b, ~a, ~b", true),
            ("|- a par b, ~a, ~b", false),
            ("|- (a * b) par (~a * ~b)", false),
            ("|- a * ~a", false),
            ("|- a par ~a", true),
            ("|- a * b, ~a * ~b", false),
            ("|- a * b, c * (~a par ~b), ~c", true),
            ("a, a -o b, b -o c |- c", true),
            ("a -o b, b -o c |- a -o c", true),
            ("a -o b |- b -o a", false),
            ("|- (a par ~a) par (b par ~b)", false),
            ("|- a, a, ~a * ~a", true),
            ("|- a, a, a, ~a * (~a * ~a)", true),
            ("|- a, ~a, a, ~a", false),
        ] {
            assert_eq!(provable(input, m), expected, "{input:?}");
        }
        let mix = m.with_mix();
        for (input, expected) in [
            ("|-", false),
            ("|- a, ~a, a, ~a", true),
            ("|- a par b, ~a, ~b", true),
            ("|- a * b, ~a * ~b", false),
            ("|- a * ~a", false),
            ("|- (a par ~a) par (b par ~b)", true),
            ("|- a, a, ~a * ~a", true),
            ("|- a, a, ~a, ~a, b par c, ~b, ~c", true),
        ] {
            assert_eq!(provable(input, mix), expected, "{input:?} with Mix");
        }
    }

    /// A sequent whose atoms are all distinct is decided without any
    /// backtracking: one node and one link per atom, and one exact test
    /// per link on a small structure.
    #[test]
    fn distinct_atoms_need_no_backtracking() {
        let k = 40;
        let atoms: Vec<String> = (0..k).map(|i| format!("a{i}")).collect();
        let tensors = atoms.join(" * ");
        let pars: Vec<String> = atoms.iter().map(|a| format!("~{a}")).collect();
        let input = format!("|- {tensors}, {}", pars.join(" par "));
        let (verdict, statistics) = run(&input, Mode::CLASSICAL, &Options::default());
        assert!(verdict.proof().is_some());
        assert_eq!(statistics.nodes, k as u64);
        assert_eq!(statistics.links, k as u64);
        assert_eq!(statistics.tests, k as u64);
        // A larger structure is tested every fourth link and at the end.
        let (verdict, statistics) = run(
            &input,
            Mode::CLASSICAL,
            &Options::default().test_period(Some(7)),
        );
        assert!(verdict.proof().is_some());
        assert_eq!(statistics.links, k as u64);
        assert_eq!(statistics.tests, (k / 7 + 1) as u64);
        let long: Vec<String> = (0..120).map(|i| format!("b{i}")).collect();
        let input = format!(
            "|- {}, {}",
            long.join(" * "),
            long.iter()
                .map(|a| format!("~{a}"))
                .collect::<Vec<_>>()
                .join(" par ")
        );
        let (verdict, statistics) = run(&input, Mode::CLASSICAL, &Options::default());
        assert!(verdict.proof().is_some());
        assert_eq!(statistics.links, 120);
        assert_eq!(statistics.tests, 30);
    }

    /// Equal literal conclusions are linked in order: a refutation with
    /// repeated conclusions tries each set of partners once rather than in
    /// every order, and a provable sequent with repeated conclusions is
    /// still proved.
    #[test]
    fn symmetry_of_equal_conclusions() {
        // Four `a` conclusions, and a switching cycle through `b` and `c`
        // that only the exact test sees. With the test postponed to the
        // complete linking, the four `a` are paired before it fails: once
        // in order, not in all 24 orders.
        let input = "|- a, a, a, a, ((~a * ~a) * (~a * ~a)) * (b * c), \
                     (~b par d) * (~c par f), ~d * ~f";
        let postponed = Options::default().test_period(Some(100));
        let (verdict, statistics) = run(input, Mode::CLASSICAL, &postponed);
        assert!(matches!(verdict, Verdict::Unprovable));
        assert_eq!(statistics.tests, 1);
        assert!(statistics.links < 40, "{statistics:?}");
        let (verdict, statistics) = run(input, Mode::CLASSICAL, &Options::default());
        assert!(matches!(verdict, Verdict::Unprovable));
        assert_eq!(statistics.links, 2, "the cycle is seen at the second link");
        let provable_twin = "|- a, a, a, a, ((~a * ~a) * (~a * ~a)) * (b * c), \
                             (~b par ~c) * (d * f), ~d, ~f";
        assert!(provable(provable_twin, Mode::CLASSICAL));
    }

    /// The stop condition ends the search with `Unknown` at the next node.
    #[test]
    fn stop() {
        let s: Sequent = "|- a * b, ~a, ~b".parse().unwrap();
        let forest = Forest::new(&s).unwrap();
        let mut polls = 0;
        let (verdict, statistics, net) =
            search(&forest, Mode::CLASSICAL, &Options::default(), &mut || {
                polls += 1;
                polls == 2
            });
        assert!(matches!(verdict, Verdict::Unknown(Reason::Stopped)));
        assert!(net.is_none());
        assert_eq!(statistics.nodes, 2);
        assert_eq!(statistics.links, 1);
    }

    /// Decides `input` with the focused engine.
    fn focus_verdict(s: &Sequent, mode: Mode) -> bool {
        let forest = Forest::new(s).unwrap();
        let (verdict, _) = focus::search(
            &forest,
            s.fragment(),
            mode,
            None,
            &Options::default(),
            &mut || false,
        );
        match verdict {
            Verdict::Proved(proof) => {
                assert_eq!(proof.check(mode), Ok(()));
                true
            }
            Verdict::Unprovable => false,
            Verdict::Unknown(reason) => panic!("{s}: the focused engine says {reason}"),
        }
    }

    /// Decides `input` with the net engine, checking as `run` does.
    fn net_verdict(text: &str, mode: Mode, options: &Options) -> bool {
        match run(text, mode, options).0 {
            Verdict::Proved(_) => true,
            Verdict::Unprovable => false,
            Verdict::Unknown(reason) => panic!("{text}: {reason}"),
        }
    }

    /// The sample the two engines are compared on: generated provable
    /// sequents, mutants of them, doubled sequents (equal conclusions),
    /// and random sequents that pass the counts; with and without Mix.
    /// Returns the texts with their modes.
    pub(super) fn sample(samples: usize, budget: usize, pairs: usize) -> Vec<(String, Mode)> {
        let mut texts = Vec::new();
        for mix in [false, true] {
            let mode = mode_for(mix);
            let rules = Rules {
                units: false,
                additives: false,
                mix,
                exponentials: false,
            };
            let mut rng = Rng::new(u64::from(mix) + 10);
            for _ in 0..samples {
                let budget = 2 + rng.below(budget - 1);
                let mut formulas = generate::provable(&mut rng, rules, 3, budget).formulas;
                texts.push((generate::sequent(&formulas), mode));
                let doubled: Vec<_> = formulas.iter().chain(&formulas).cloned().collect();
                texts.push((generate::sequent(&doubled), mode));
                if generate::mutate(&mut rng, &mut formulas, 3) {
                    texts.push((generate::sequent(&formulas), mode));
                }
                let pairs = 1 + rng.below(pairs);
                let atoms = 1 + rng.below(3) as u8;
                let balanced = generate::balanced(&mut rng, atoms, pairs, mix);
                texts.push((generate::sequent(&balanced), mode));
            }
        }
        texts
    }

    /// Compares the two engines on a sample and returns how many sequents
    /// were decided, how many were provable, and the time each engine
    /// took.
    fn compare(
        sample: &[(String, Mode)],
        period: Option<u32>,
    ) -> (usize, usize, Duration, Duration) {
        let options = Options::default().test_period(period);
        let (mut provable, mut net_time, mut focus_time) = (0, Duration::ZERO, Duration::ZERO);
        for (text, mode) in sample {
            let s: Sequent = text.parse().unwrap();
            let start = Instant::now();
            let by_focus = focus_verdict(&s, *mode);
            focus_time += start.elapsed();
            let start = Instant::now();
            let by_net = net_verdict(text, *mode, &options);
            net_time += start.elapsed();
            assert_eq!(
                by_net, by_focus,
                "{text:?} in {mode} mode: net {by_net}, focus {by_focus}"
            );
            provable += usize::from(by_net);
        }
        (sample.len(), provable, net_time, focus_time)
    }

    /// The net engine agrees with the focused engine on every sequent of
    /// the sample, at the default cadence of the exact test and when it
    /// runs only every third link.
    #[test]
    fn agrees_with_the_focused_engine() {
        let sample = sample(60, 12, 8);
        let (decided, provable, _, _) = compare(&sample, None);
        assert_eq!(decided, sample.len());
        assert!(
            provable > 100 && provable < decided - 100,
            "{provable} of {decided}"
        );
        compare(&sample, Some(3));
    }

    /// The same on a larger sample, with the time each engine took; run it
    /// in release mode and read the numbers it prints.
    #[test]
    #[ignore = "a larger sample; run with --release -- --ignored --nocapture"]
    fn net_versus_focus_timing() {
        let sample = sample(150, 16, 10);
        let (decided, provable, net_time, focus_time) = compare(&sample, None);
        println!(
            "{decided} sequents decided alike, {provable} provable; net {net_time:.2?}, focus {focus_time:.2?}"
        );
        let (_, _, net_time, _) = compare(&sample, Some(4));
        println!("net with the exact test every fourth link: {net_time:.2?}");
    }

    /// Small Partition instances are decided as the instance says, by both
    /// engines. The equal literals inside `b ⊗ b ⊗ …` and `~b ⅋ ~b` are
    /// interchangeable, so the net engine visits a symmetric subtree for
    /// every wrong choice and falls far behind the focused engine here.
    #[test]
    fn partition_instances() {
        for (sizes, expected) in [
            (&[1, 1][..], true),
            (&[1, 3], false),
            (&[2, 1, 1], true),
            (&[1, 1, 4], false),
        ] {
            let s = crate::families::partition(sizes);
            assert_eq!(
                provable(&s.to_string(), Mode::CLASSICAL),
                expected,
                "{sizes:?}"
            );
            assert_eq!(focus_verdict(&s, Mode::CLASSICAL), expected, "{sizes:?}");
        }
    }

    /// The engine is deterministic: the same input gives the same proof
    /// and the same counters.
    #[test]
    fn deterministic() {
        let input = "|- a, a, ~a * (b par ~b), ~a * (c * ~c) par d, ~d";
        let (first, stats) = run(input, Mode::CLASSICAL.with_mix(), &Options::default());
        let (second, again) = run(input, Mode::CLASSICAL.with_mix(), &Options::default());
        assert_eq!(stats, again);
        assert_eq!(
            first.proof().map(|p| p.nodes().to_vec()),
            second.proof().map(|p| p.nodes().to_vec())
        );
        assert_eq!(Engine::Net.to_string(), "net");
    }
}

/// The net engine on several threads: cube-and-conquer over the first
/// links.
#[cfg(feature = "parallel")]
pub(crate) mod parallel {
    use super::{Engine, counts_admit};
    use crate::fragment::Mode;
    use crate::nets::ProofStructure;
    use crate::occurrences::{Forest, OccId};
    use crate::search::parallel::Runtime;
    use crate::search::{Options, Reason, Statistics, Stop, Verdict};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    /// How many cubes per thread the enumeration aims for, so that the
    /// cubes that turn out large are shared out among many small ones.
    const CUBES_PER_THREAD: usize = 16;

    /// A cube: the first links of a branch of the search, in the order
    /// they were made.
    type Cube = Vec<(OccId, OccId)>;

    /// What the workers of a cube-and-conquer run report.
    struct Collected {
        /// The first proof net found.
        net: Option<ProofStructure>,
        /// The first error, a stop that a cancellation caused excepted,
        /// a stop giving way to any other reason.
        error: Option<Reason>,
        /// The counters of every engine that ran.
        statistics: Statistics,
    }

    /// Runs the net engine on the runtime's pool, as [`super::search`]
    /// does on one thread, polling `stop` on the calling thread while the
    /// pool searches. The root engine enumerates the branches of the
    /// first `d` links as cubes, `d` growing until there are
    /// [`CUBES_PER_THREAD`] cubes per thread or the enumeration decided
    /// the sequent by itself; then every worker takes cubes from a
    /// shared counter, one engine of its own reset per cube, until a
    /// proof net is found, which stops the others, or the cubes run out.
    /// `Unprovable` needs every cube to have been searched to its end.
    pub(crate) fn search(
        forest: &Forest,
        mode: Mode,
        options: &Options,
        runtime: &Runtime,
        stop: &mut dyn FnMut() -> bool,
    ) -> (Verdict, Statistics, Option<ProofStructure>) {
        if !counts_admit(forest, mode.mix) {
            return (Verdict::Unprovable, Statistics::default(), None);
        }
        let threads = runtime.threads();
        let pairs = forest.all_literals().len() / 2;
        let (result, statistics, net) = runtime.drive(stop, |flags| {
            let mut root = Engine::new(forest, mode, options, Stop::Flags(flags));
            let mut cubes = Vec::new();
            let mut depth = 1;
            loop {
                cubes.clear();
                match root.explore(Some(depth), &mut cubes) {
                    Ok(true) => return (Ok(true), root.statistics, Some(root.net)),
                    Ok(false) if cubes.is_empty() => return (Ok(false), root.statistics, None),
                    Ok(false) => {}
                    Err(reason) => return (Err(reason), root.statistics, None),
                }
                if cubes.len() >= CUBES_PER_THREAD * threads || depth >= pairs {
                    break;
                }
                root.reset();
                depth += 1;
            }
            let next = AtomicUsize::new(0);
            let found = AtomicBool::new(false);
            let collected = Mutex::new(Collected {
                net: None,
                error: None,
                statistics: root.statistics,
            });
            runtime.pool.scope(|scope| {
                for _ in 0..threads {
                    let (cubes, next, found, collected, flags) =
                        (&cubes, &next, &found, &collected, &flags);
                    scope.spawn(move |_| {
                        let mut engine =
                            Engine::new(forest, mode, options, Stop::Flags(flags.child(found)));
                        loop {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            let Some(cube) = cubes.get(i) else {
                                break;
                            };
                            engine.reset();
                            engine.seed(cube);
                            match engine.run() {
                                Ok(false) => continue,
                                Ok(true) => {
                                    let mut collected = lock(collected);
                                    if collected.net.is_none() {
                                        collected.net = Some(engine.net.clone());
                                    }
                                    found.store(true, Ordering::Relaxed);
                                }
                                Err(Reason::Stopped) if found.load(Ordering::Relaxed) => {}
                                Err(reason) => {
                                    let mut collected = lock(collected);
                                    if collected.error.is_none_or(|old| old == Reason::Stopped) {
                                        collected.error = Some(reason);
                                    }
                                    found.store(true, Ordering::Relaxed);
                                }
                            }
                            break;
                        }
                        lock(collected).statistics.add(&engine.statistics);
                    });
                }
            });
            let collected = collected
                .into_inner()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            match (collected.net, collected.error) {
                (Some(net), _) => (Ok(true), collected.statistics, Some(net)),
                (None, Some(reason)) => (Err(reason), collected.statistics, None),
                (None, None) => (Ok(false), collected.statistics, None),
            }
        });
        let verdict = match result {
            Ok(true) => {
                let net = net.as_ref().expect("a proof net was found");
                let proof = net
                    .sequentialize()
                    .expect("a complete linking that passed the exact test is a proof net");
                debug_assert_eq!(proof.check(mode), Ok(()), "the engine's proof");
                Verdict::Proved(Box::new(proof))
            }
            Ok(false) => Verdict::Unprovable,
            Err(reason) => Verdict::Unknown(reason),
        };
        (verdict, statistics, net)
    }

    /// Locks what the workers report.
    fn lock<T>(shared: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
        shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(all(test, feature = "parse", feature = "parallel"))]
mod parallel_tests {
    use super::tests::sample;
    use crate::fragment::Mode;
    use crate::search::{Engine, Options, Reason, Verdict, prove, prove_until};
    use crate::sequents::Sequent;

    /// The verdict of the net engine on `jobs` threads, the proof and the
    /// net checked.
    fn verdict(text: &str, mode: Mode, jobs: usize) -> Verdict {
        let sequent: Sequent = text.parse().unwrap();
        let options = Options::default().engine(Some(Engine::Net)).jobs(jobs);
        let outcome = prove(&sequent, mode, &options).unwrap();
        if let Verdict::Proved(proof) = &outcome.verdict {
            assert_eq!(proof.check(mode), Ok(()), "{text:?}");
            let net = outcome.net.as_ref().expect("the net found");
            assert_eq!(net.is_correct(), Ok(()), "{text:?}");
        }
        outcome.verdict
    }

    /// The net engine on two and four threads agrees with itself on one
    /// on generated sequents, provable and balanced ones, in both modes.
    #[test]
    fn cubes_agree_with_the_sequential_search() {
        for (text, mode) in sample(20, 6, 6) {
            let sequential = verdict(&text, mode, 1);
            for jobs in [2, 4] {
                let parallel = verdict(&text, mode, jobs);
                assert_eq!(
                    matches!(sequential, Verdict::Proved(_)),
                    matches!(parallel, Verdict::Proved(_)),
                    "{text:?} in {mode} mode: {sequential:?} on one thread, {parallel:?} on {jobs}"
                );
                assert_eq!(
                    matches!(sequential, Verdict::Unprovable),
                    matches!(parallel, Verdict::Unprovable),
                    "{text:?} in {mode} mode: {sequential:?} on one thread, {parallel:?} on {jobs}"
                );
            }
        }
    }

    /// A stop condition that fires at once stops the pool: the driver
    /// raises the flag at its first poll, a millisecond in, long before
    /// the refutation of this Partition instance ends (seconds on one
    /// thread, half a second on eight).
    #[test]
    fn stops() {
        let sequent = crate::families::partition(&[1, 2, 5]);
        let options = Options::default().engine(Some(Engine::Net)).jobs(2);
        let outcome = prove_until(&sequent, Mode::CLASSICAL, &options, || true).unwrap();
        assert!(
            matches!(outcome.verdict, Verdict::Unknown(Reason::Stopped)),
            "{:?}",
            outcome.verdict
        );
    }
}
