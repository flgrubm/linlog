// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Problem families with known verdicts, generated at any size: the hard
//! families of the literature (3-Partition as a Horn program and as
//! Lincoln and Winkler's two-literal sequent, Matsuoka's Partition, random
//! QBF under a lock-and-key encoding into MALL, Petri-net counters), the
//! families where one engine is known to be slow (wide contexts under
//! Mix, wide sequents of distinct or shared atoms), and the slow cases of
//! the exponentials. Each instance says whether it is provable, by
//! construction or by solving the combinatorial problem it encodes, never
//! by running an engine. Random families are seeded by their size and
//! index, so an instance is the same on every run.
//!
//! # Examples
//!
//! ```
//! use linlog::families;
//! use linlog::search::{Options, Verdict, prove};
//!
//! let family = families::find("partition-no").unwrap();
//! let instance = family.instance(3, 0);
//! assert!(!instance.provable);
//! let outcome = prove(&instance.sequent, instance.mode, &Options::default())?;
//! assert!(matches!(outcome.verdict, Verdict::Unprovable));
//! # Ok::<(), linlog::Error>(())
//! ```

use crate::{Mode, Sequent};

/// One problem of a family.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Instance {
    /// The family's name, the size and, for random families, the index:
    /// `qbf/8#2`.
    pub name: String,
    /// The sequent.
    pub sequent: Sequent,
    /// The mode the sequent is meant for.
    pub mode: Mode,
    /// Whether the sequent is provable in its mode.
    pub provable: bool,
    /// The per-branch copy bound under which a proof exists, for the
    /// families with exponentials; `None` for the others.
    pub copies: Option<u32>,
}

/// A family of problems, indexed by a size.
#[derive(Debug)]
#[non_exhaustive]
pub struct Family {
    /// The name, as [`find`] takes it.
    pub name: &'static str,
    /// What the family is and what its size counts.
    pub summary: &'static str,
    /// The sizes a benchmark runs by default: from easy to past what the
    /// engines decide in minutes.
    pub sizes: &'static [u32],
    /// How many instances each size has: one for the constructed families,
    /// several for the random ones.
    pub instances: u32,
    /// Builds the instance of a size and an index.
    generate: fn(u32, u32) -> Instance,
}

impl Family {
    /// Returns the instance of the given size and index (below
    /// [`instances`](Self::instances)), the same on every call.
    ///
    /// # Panics
    ///
    /// For a size the family has no instance of: a 3-Partition bin smaller
    /// than four, a QBF of fewer than three variables, a solvable Partition
    /// of fewer than two items or an unsolvable one of fewer than three, a
    /// counter whose tokens are not a power of two.
    pub fn instance(&self, size: u32, index: u32) -> Instance {
        let mut instance = (self.generate)(size, index);
        instance.name = if self.instances > 1 {
            format!("{}/{size}#{index}", self.name)
        } else {
            format!("{}/{size}", self.name)
        };
        instance
    }
}

/// Every family, in the order a benchmark runs them.
pub static FAMILIES: &[Family] = &[
    Family {
        name: "3-partition-yes",
        summary: "3-Partition as a Horn program with a `&` over the bins per item (MALL), two bins of the size, solvable",
        sizes: &[4, 6, 8, 12],
        instances: 1,
        generate: |b, _| {
            let items = solvable_triples(b);
            instance(three_partition(&items, 2, b), Mode::CLASSICAL, true)
        },
    },
    Family {
        name: "3-partition-no",
        summary: "3-Partition as a Horn program with a `&` over the bins per item (MALL), two bins of the size, unsolvable",
        sizes: &[4, 5],
        instances: 1,
        generate: |b, _| {
            instance(
                three_partition(&unsolvable_triples(b), 2, b),
                Mode::CLASSICAL,
                false,
            )
        },
    },
    Family {
        name: "3-partition-mll-yes",
        summary: "3-Partition as Lincoln and Winkler's two-literal MLL sequent, two bins of the size, solvable",
        sizes: &[4, 6, 8, 12],
        instances: 1,
        generate: |b, _| {
            let items = solvable_triples(b);
            instance(three_partition_mll(&items, 2, b), Mode::CLASSICAL, true)
        },
    },
    Family {
        name: "3-partition-mll-no",
        summary: "3-Partition as Lincoln and Winkler's two-literal MLL sequent, two bins of the size, unsolvable",
        sizes: &[4, 5, 6, 8],
        instances: 1,
        generate: |b, _| {
            instance(
                three_partition_mll(&unsolvable_triples(b), 2, b),
                Mode::CLASSICAL,
                false,
            )
        },
    },
    Family {
        name: "partition-yes",
        summary: "Matsuoka's Horn encoding of Partition (MLL) with this many random items, solvable",
        sizes: &[4, 5, 6, 7],
        instances: 1,
        generate: |n, _| instance(partition(&partition_items(n, true)), Mode::CLASSICAL, true),
    },
    Family {
        name: "partition-no",
        summary: "Matsuoka's Horn encoding of Partition (MLL) with this many random items, unsolvable",
        sizes: &[3, 4, 5],
        instances: 1,
        generate: |n, _| {
            instance(
                partition(&partition_items(n, false)),
                Mode::CLASSICAL,
                false,
            )
        },
    },
    Family {
        name: "qbf",
        summary: "random 3-QBF with this many alternating variables (∃ first), 1.5 clauses per variable with two existentials each, under a lock-and-key encoding into MALL",
        sizes: &[8, 12, 16, 20, 24],
        instances: 4,
        generate: |n, index| {
            let (formula, valid) = qbf(n, index);
            instance(formula, Mode::CLASSICAL, valid)
        },
    },
    Family {
        name: "wide-m1",
        summary: "(a1 ⅋ b1) ⊗ … ⊗ (ak ⅋ bk), ~a1 ⊗ ~b1, …, ~ak ⊗ ~bk over distinct atoms (MLL), provable; the size is k",
        sizes: &[8, 16, 24, 32, 256, 2048],
        instances: 1,
        generate: |k, _| instance(wide(k, 1), Mode::CLASSICAL, true),
    },
    Family {
        name: "wide-m2",
        summary: "the wide sequent with every atom shared by two pairs: every literal twice",
        sizes: &[8, 16, 24, 32, 256, 2048],
        instances: 1,
        generate: |k, _| instance(wide(k, 2), Mode::CLASSICAL, true),
    },
    Family {
        name: "wide-m3",
        summary: "the wide sequent with every atom shared by three pairs: every literal three times",
        sizes: &[12, 24, 30, 36],
        instances: 1,
        generate: |k, _| instance(wide(k, 3), Mode::CLASSICAL, true),
    },
    Family {
        name: "wide-m4",
        summary: "the wide sequent with every atom shared by four pairs: every literal four times",
        sizes: &[12, 24, 28, 32, 36],
        instances: 1,
        generate: |k, _| instance(wide(k, 4), Mode::CLASSICAL, true),
    },
    Family {
        name: "mix",
        summary: "(ai ⊗ bi) ⊕ 0, (~ai ⊗ ~bi) ⊕ 0 for this many i under Mix (MALL, where no count equation refutes it at once), unprovable",
        sizes: &[4, 6, 8, 9, 10, 11],
        instances: 1,
        generate: |k, _| instance(mix(k), Mode::CLASSICAL.with_mix(), false),
    },
    Family {
        name: "counter",
        summary: "the Petri-net counter !(c0 ⊗ c0 ⊸ c1), …, c0^n ⊢ cL with n tokens, n a power of two (MELL), provable within log2 n copies",
        sizes: &[2, 4, 8, 16],
        instances: 1,
        generate: |n, _| {
            let (sequent, levels) = counter(n, false);
            Instance {
                copies: Some(levels),
                ..instance(sequent, Mode::CLASSICAL, true)
            }
        },
    },
    Family {
        name: "counter-over",
        summary: "the counter with the unreachable goal cL ⊗ c0 (MELL), unprovable, searched to log2 n copies",
        sizes: &[2, 4, 8, 16],
        instances: 1,
        generate: |n, _| {
            let (sequent, levels) = counter(n, true);
            Instance {
                copies: Some(levels),
                ..instance(sequent, Mode::CLASSICAL, false)
            }
        },
    },
    Family {
        name: "growing",
        summary: "!(a ⊸ a ⊗ a), a ⊢ ?b (MELL), unprovable, a context that grows with every copy; the size is the copy bound",
        sizes: &[16, 64, 256, 1024],
        instances: 1,
        generate: |copies, _| Instance {
            copies: Some(copies),
            ..instance(parse("!(a -o a * a), a |- ?b"), Mode::CLASSICAL, false)
        },
    },
    Family {
        name: "chain",
        summary: "!(p0 ⊸ p1 & p2), …, !(p(k−1) ⊸ pk & p(k+1)), p0 ⊢ p(k+1) (MELL, ILL-shaped), provable within k + 2 copies; the size is k",
        sizes: &[16, 64, 128, 256],
        instances: 1,
        generate: |k, _| {
            let clauses: Vec<String> = (0..k)
                .map(|i| format!("!(p{i} -o p{} & p{})", i + 1, i + 2))
                .collect();
            let text = format!("{}, p0 |- p{}", clauses.join(", "), k + 1);
            Instance {
                copies: Some(k + 2),
                ..instance(parse(&text), Mode::CLASSICAL, true)
            }
        },
    },
    Family {
        name: "additive",
        summary: "A ⊢ A for A a complete tree of this depth alternating & and ⊕ over distinct atoms (the additive path), provable",
        sizes: &[8, 12, 14, 16],
        instances: 1,
        generate: |depth, _| {
            let formula = additive_tree(depth, 0, &mut 0);
            instance(
                parse(&format!("{formula} |- {formula}")),
                Mode::CLASSICAL,
                true,
            )
        },
    },
];

/// Returns the family of that name.
pub fn find(name: &str) -> Option<&'static Family> {
    FAMILIES.iter().find(|family| family.name == name)
}

/// Encodes a 3-Partition instance as a linear Horn program in the style of
/// Kanovich's encodings: bin `j` offers `size` units `bj` and three slots
/// `tj`; item `i` is a `&` over the bins of a clause taking its units and
/// a slot from that bin and producing `di`; the goal is the tensor of every
/// `di`. Every resource must be used exactly once, so the sequent is
/// provable if and only if the items split into triples of sum `size`,
/// one per bin.
pub fn three_partition(items: &[u32], bins: u32, size: u32) -> Sequent {
    let mut hypotheses = Vec::new();
    for j in 1..=bins {
        hypotheses.extend((0..size).map(|_| format!("b{j}")));
        hypotheses.extend((0..3).map(|_| format!("t{j}")));
    }
    for (i, &a) in items.iter().enumerate() {
        let clauses: Vec<String> = (1..=bins)
            .map(|j| format!("({} * t{j} -o d{i})", power(&format!("b{j}"), a, " * ")))
            .collect();
        hypotheses.push(clauses.join(" & "));
    }
    let goal: Vec<String> = (0..items.len()).map(|i| format!("d{i}")).collect();
    parse(&format!(
        "{} |- {}",
        hypotheses.join(", "),
        goal.join(" * ")
    ))
}

/// Encodes a 3-Partition instance as Lincoln and Winkler's two-literal MLL
/// sequent `⊢ k ⊗ (~c ⅋ … ⅋ ~c), …, ((~k ⅋ ~k ⅋ ~k) ⅋ (c ⊗ … ⊗ c)) ⊗ …`:
/// an item of size `s` is `k ⊗` a par of `s` literals `~c`, a bin is three
/// `~k` beside a tensor of `size` literals `c`, and the bins are joined by
/// `⊗`, so that each bin's premise takes three items of total `size`.
pub fn three_partition_mll(items: &[u32], bins: u32, size: u32) -> Sequent {
    let mut roots: Vec<String> = items
        .iter()
        .map(|&s| format!("k * ({})", power("~c", s, " | ")))
        .collect();
    let bin = format!("((~k | ~k | ~k) | ({}))", power("c", size, " * "));
    roots.push(power(&bin, bins, " * "));
    parse(&format!("|- {}", roots.join(", ")))
}

/// Encodes a Partition instance as a Horn sequent, as Matsuoka does: every
/// item `a_i` of size `s_i` may be turned into `s_i` units `b` or `s_i`
/// units `c`; half the total of each buys the items back once, and half
/// the total of each buys the goal `e`. Every clause is used exactly once,
/// so the sequent is provable if and only if the sizes split into two
/// halves of equal sum.
///
/// # Panics
///
/// If the sizes add up to an odd total.
pub fn partition(sizes: &[u32]) -> Sequent {
    let total: u32 = sizes.iter().sum();
    assert_eq!(total % 2, 0, "the total is even");
    let items: Vec<String> = (1..=sizes.len()).map(|i| format!("a{i}")).collect();
    let mut hypotheses = vec![items.join(" * ")];
    for unit in ["b", "c"] {
        for (i, &size) in sizes.iter().enumerate() {
            hypotheses.push(format!("(a{} -o {})", i + 1, power(unit, size, " * ")));
        }
    }
    let half = format!(
        "({} * {})",
        power("b", total / 2, " * "),
        power("c", total / 2, " * ")
    );
    hypotheses.push(format!("({half} -o {})", items.join(" * ")));
    hypotheses.push(format!("({half} -o e)"));
    parse(&format!("{} |- e", hypotheses.join(", ")))
}

/// The counter program as Petri-net reachability: `tokens` tokens `c0`,
/// and reusable clauses `!(ci ⊗ ci ⊸ c(i+1))` doubling up to the goal
/// `cL`, `L` the logarithm of `tokens`; with `over`, the goal is `cL ⊗
/// c0`, which no firing reaches. Returns the sequent and `L`, the copies
/// a branch to one token of the goal takes.
///
/// # Panics
///
/// If `tokens` is not a power of two.
pub fn counter(tokens: u32, over: bool) -> (Sequent, u32) {
    assert!(tokens.is_power_of_two(), "{tokens} tokens");
    let levels = tokens.trailing_zeros();
    let mut hypotheses: Vec<String> = (0..levels)
        .map(|i| format!("!(c{i} * c{i} -o c{})", i + 1))
        .collect();
    hypotheses.extend((0..tokens).map(|_| "c0".to_owned()));
    let extra = if over { " * c0" } else { "" };
    let text = format!("{} |- c{levels}{extra}", hypotheses.join(", "));
    (parse(&text), levels)
}

/// The wide sequent `⊢ (a1 ⅋ b1) ⊗ … ⊗ (ak ⅋ bk), ~a1 ⊗ ~b1, …, ~ak ⊗ ~bk`
/// with pair `i` using the atoms of index `i / shared`, so that every
/// literal occurs `shared` times (the last atoms fewer when `shared` does
/// not divide `k`). Provable: identifying atoms keeps a proof a proof.
pub fn wide(k: u32, shared: u32) -> Sequent {
    let pairs: Vec<String> = (0..k)
        .map(|i| format!("(a{0} | b{0})", i / shared))
        .collect();
    let mut roots = vec![pairs.join(" * ")];
    roots.extend((0..k).map(|i| format!("~a{0} * ~b{0}", i / shared)));
    parse(&format!("|- {}", roots.join(", ")))
}

/// The unprovable sequent `⊢ (a1 ⊗ b1) ⊕ 0, (~a1 ⊗ ~b1) ⊕ 0, …` of `k`
/// pairs: every axiom linking of a pair closes a cycle through its two
/// `⊗`, with Mix too, and the `⊕ 0` makes it MALL, where the count
/// equation that refutes the bare tensors at once does not hold, so that
/// a search under Mix meets the partitions of the context.
pub fn mix(k: u32) -> Sequent {
    let roots: Vec<String> = (0..k)
        .map(|i| format!("(a{i} * b{i}) + 0, (~a{i} * ~b{i}) + 0"))
        .collect();
    parse(&format!("|- {}", roots.join(", ")))
}

/// A random closed 3-QBF and its truth value: `n` variables quantified
/// alternately, `∃` first, and `⌈1.5 n⌉` clauses of three distinct
/// variables of which at least two are existential (Gent and Walsh's
/// model A, which keeps random instances from being false outright), with
/// random signs; seeded by `n` and `index`. The formula is encoded into
/// MALL by locks and keys: a `∀x` is `(~tx & ~fx) ⅋ S`, putting one of the
/// two values into the context on each premise; an `∃x` is `((~tx ⅋ ~kx)
/// ⊕ (~fx ⅋ ~kx)) ⅋ (kx ⊗ S)`, whose rest `S` needs the key `~kx` that
/// only the choice of a value releases, so the choice precedes every
/// later quantifier; the matrix is the `&` of the clauses, a clause the
/// `⊕` of `(tx ⊗ ⊤)` or `(fx ⊗ ⊤)` over its literals, provable when the
/// context holds a value that satisfies one of them.
///
/// # Panics
///
/// For fewer than three variables, which leave no clause to draw.
pub fn qbf(n: u32, index: u32) -> (Sequent, bool) {
    assert!(n >= 3, "a QBF of {n} variables has no clause of three");
    let mut rng = Rng((u64::from(n) << 32) | u64::from(index));
    let existential = |v: u32| v.is_multiple_of(2);
    let mut clauses: Vec<[(u32, bool); 3]> = Vec::new();
    while clauses.len() < (3 * n).div_ceil(2) as usize {
        let x = rng.below(n);
        let y = rng.below(n);
        let z = rng.below(n);
        let vars = [x, y, z];
        if x == y || y == z || x == z || vars.iter().filter(|&&v| existential(v)).count() < 2 {
            continue;
        }
        clauses.push(vars.map(|v| (v, rng.below(2) == 0)));
    }
    let valid = qbf_value(n, &clauses, &mut Vec::new());

    let literal =
        |(v, positive): (u32, bool)| format!("({}{v} * top)", if positive { "t" } else { "f" });
    let matrix: Vec<String> = clauses
        .iter()
        .map(|clause| format!("({})", clause.map(literal).join(" + ")))
        .collect();
    let mut formula = format!("({})", matrix.join(" & "));
    for v in (0..n).rev() {
        formula = if existential(v) {
            format!("(((~t{v} | ~k{v}) + (~f{v} | ~k{v})) | (k{v} * {formula}))")
        } else {
            format!("((~t{v} & ~f{v}) | {formula})")
        };
    }
    (parse(&format!("|- {formula}")), valid)
}

/// Evaluates the QBF of [`qbf`] under the values of the first variables.
fn qbf_value(n: u32, clauses: &[[(u32, bool); 3]], values: &mut Vec<bool>) -> bool {
    let v = values.len() as u32;
    if v == n {
        return clauses
            .iter()
            .all(|clause| clause.iter().any(|&(x, sign)| values[x as usize] == sign));
    }
    let mut outcomes = [true, false].into_iter().map(|value| {
        values.push(value);
        let result = qbf_value(n, clauses, values);
        values.pop();
        result
    });
    if v.is_multiple_of(2) {
        outcomes.any(|b| b)
    } else {
        outcomes.all(|b| b)
    }
}

/// Six item sizes for two bins of size `b` (at least four) that split into
/// the triples `1, 2, b − 3`.
fn solvable_triples(b: u32) -> [u32; 6] {
    assert!(b >= 4, "bins of {b}");
    [1, 2, b - 3, 1, 2, b - 3]
}

/// Six item sizes for two bins of size `b` (at least four) that admit no
/// 3-Partition: the item `b − 1` needs two more items of total one.
fn unsolvable_triples(b: u32) -> [u32; 6] {
    assert!(b >= 4, "bins of {b}");
    [1, 1, 1, b - 1, 1, b - 3]
}

/// Returns `n` random item sizes from 1 to `n` with an even total whose
/// Partition instance is solvable or not as asked: the first seed, counted
/// from one derived from `n` and `solvable`, whose sizes answer so.
fn partition_items(n: u32, solvable: bool) -> Vec<u32> {
    // Two items of sizes one or two always split, one never does.
    assert!(n >= 2 + u32::from(!solvable), "{n} items");
    let mut rng = Rng((u64::from(n) << 1) | u64::from(solvable));
    loop {
        let sizes: Vec<u32> = (0..n).map(|_| 1 + rng.below(n)).collect();
        let total: u32 = sizes.iter().sum();
        if total % 2 == 1 {
            continue;
        }
        // Subset sums, as a bit per reachable sum.
        let mut reachable = vec![false; total as usize + 1];
        reachable[0] = true;
        for &s in &sizes {
            for sum in (s..=total).rev() {
                reachable[sum as usize] |= reachable[(sum - s) as usize];
            }
        }
        if reachable[total as usize / 2] == solvable {
            return sizes;
        }
    }
}

/// A complete tree of the given depth over the atoms `x<next>`, `&` at even
/// levels and `⊕` at odd ones.
fn additive_tree(depth: u32, level: u32, next: &mut u32) -> String {
    if depth == level {
        *next += 1;
        return format!("x{}", *next - 1);
    }
    let left = additive_tree(depth, level + 1, next);
    let right = additive_tree(depth, level + 1, next);
    let connective = if level.is_multiple_of(2) { "&" } else { "+" };
    format!("({left} {connective} {right})")
}

/// `n` copies of `formula` joined by `separator`.
fn power(formula: &str, n: u32, separator: &str) -> String {
    vec![formula; n as usize].join(separator)
}

/// Parses a sequent the generators built.
fn parse(text: &str) -> Sequent {
    text.parse()
        .unwrap_or_else(|e| panic!("a generated sequent parses: {e}"))
}

/// An instance with a name to be filled in by [`Family::instance`].
fn instance(sequent: Sequent, mode: Mode, provable: bool) -> Instance {
    Instance {
        name: String::new(),
        sequent,
        mode,
        provable,
        copies: None,
    }
}

/// SplitMix64: a small seeded generator, so that a random instance is a
/// function of its seed.
struct Rng(u64);

impl Rng {
    /// Returns a number below `n`.
    fn below(&mut self, n: u32) -> u32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        ((z ^ (z >> 31)) % u64::from(n)) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::{Options, Verdict, prove};

    /// The smallest instances of every family are decided as the family
    /// says, and every proof checks; the random families are the same on
    /// every call. The unsolvable Horn 3-Partition is left out: refuting
    /// its smallest instance takes a minute, which is what the benchmarks
    /// measure.
    #[test]
    fn verdicts_as_constructed() {
        for family in FAMILIES.iter().filter(|f| f.name != "3-partition-no") {
            for index in 0..family.instances {
                let instance = family.instance(family.sizes[0], index);
                let options =
                    Options::default().copies(instance.copies.unwrap_or(Options::DEFAULT_COPIES));
                let outcome = prove(&instance.sequent, instance.mode, &options).unwrap();
                match &outcome.verdict {
                    Verdict::Proved(proof) => {
                        assert!(instance.provable, "{} proved", instance.name);
                        proof.check(instance.mode).unwrap();
                    }
                    Verdict::Unprovable => assert!(!instance.provable, "{} refuted", instance.name),
                    // Exponential families without a proof are bounded.
                    Verdict::Unknown(_) => assert!(
                        !instance.provable && instance.copies.is_some(),
                        "{}: {:?}",
                        instance.name,
                        outcome.verdict
                    ),
                }
                assert_eq!(
                    instance.sequent,
                    family.instance(family.sizes[0], index).sequent
                );
            }
        }
    }

    /// The QBF encoding agrees with the formula's truth value on many
    /// small random instances, true and false ones alike.
    #[test]
    fn qbf_encoding() {
        let mut valid = 0;
        for n in 3..=7 {
            for index in 0..12 {
                let (sequent, expected) = qbf(n, index);
                let outcome = prove(&sequent, Mode::CLASSICAL, &Options::default()).unwrap();
                assert_eq!(
                    outcome.verdict.proof().is_some(),
                    expected,
                    "qbf({n}, {index})"
                );
                assert!(expected || matches!(outcome.verdict, Verdict::Unprovable));
                valid += usize::from(expected);
            }
        }
        assert!((10..50).contains(&valid), "{valid} of 60 valid");
    }
}
