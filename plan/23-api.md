# Step 23: the library's API and data model in order, ready for quantifiers

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/reports/17-assessment.md`: section 1 in full (1.2's refactoring
  list is this step's and step 24's and 29's; 1.3; 1.4), section 4, 3.9,
  and the author's answers 4 and 5.
- `plan/README.md`: every decision, D17 and D18 above all; the reports
  of steps 18 to 22.
- `plan/later.md`: "Code audit and refactoring", "First-order linear
  logic".
- `proof-search-specifications.md`, "First-order fragments".
- Every rules file under `.claude/rules/`, and the whole of `core/src`
  outside `search/focus/` and `search/net.rs`, whose insides are step
  29's.

## What the earlier steps left you

Sixteen steps by separate sessions built the library, each reviewed for
its own correctness and none for the whole. Step 17 read it as one
maintainer would; its list is section 1.2 of its report. Two decisions
of the author frame this step. The API may change freely until the first
release (D18): make it as good as it can be, idiomatic, ergonomic for a
caller who is not this repository, and efficient. And quantifiers will
come (D17): the data model and the interfaces are shaped so that
first-order terms, binders and substitutions can be added without a
second rewrite, and without slowing the propositional case, which both
baselines pin; if that takes generics or a duplicated fast path, so be
it.

## Goal

A library a stranger can use from its documentation, whose types say
what they hold, whose errors are one family, whose options are data with
a wire form, and whose data model has a written, checked answer to "where
do terms and binders go". No behaviour of the command changes.

## What to build

1. **A design note first**, `plan/notes/api.md`, short: the public
   surface after this step (modules, types, constructors, the handful of
   entry points), and how first-order logic enters it (atoms as
   predicates over terms, binders in the arena, a substitution beside
   the forest, witnesses in proofs), with what stays untouched for the
   propositional case and how that is measured. Decide between the
   candidates on evidence from this code, not in the abstract; a
   fresh-context reviewer reads the note before the code follows it.
2. **The surface.** What is `pub` and need not be; names that differ
   between neighbours (the three `Engine` structs beside the enum, the
   four types called `Rule` or `Rules`); `#[non_exhaustive]` and builders
   applied evenly; `Mode` where a `bool` stands for it; constructors
   that take or clone a forest by one rule; one numbering of inferences
   or a map between the two. What step 21 added is part of it:
   `Verdict::Unprovable(Refutation)`, whose `Unbalanced` holds an atom
   and its name both, which the forest has;
   `Options::copies(Option<u32>)`; `Statistics::copies`, the larger of
   two searches' levels, so 30 under a bound of 3 on a Horn program; and
   `Interactive::close_with`, which grafts any `Proof` it is handed once
   it passes the checker against the goal's ids, without asking whether
   it is a proof over the session's forest.
3. **Options with a wire form.** `search::Options` and every options
   value of the outputs serialize; the command, the batch mode and the
   web front end are then three callers of the same values (D15).
4. **One family of errors**: `ShapeError`, `Unsupported`, `UnknownRule`
   and `NetError` within reach of `Error`, one way to describe an error
   with formulas, a serializable form. Steps 18 to 20 added to the
   family and it is this step's to make them one: `ViewError`,
   `Error::Rejected`, `Error::Unchecked`, `Error::TooManyNodes`, and in
   the checker `Problem::Surplus` (a fault of the proof) beside
   `Problem::Memory` (a refusal, no verdict: `CheckError::is_refusal`).
   A caller must not be able to read a refusal as "invalid".
   And three limits that reach a caller unevenly: a proof file and a
   session's state are read under the default occurrence limit whatever
   the caller asked for, because `Deserialize` takes no options (a
   seeded form, or a constructor that takes the limit); the checker's
   pass takes a memory bound and no stop condition, though a hostile
   proof file can make it quadratic in time; and the bounds of a search,
   of a check and of a view are three values that a front end sets one
   by one.
5. **The same walk once**: one printer of formulas and of two-sided
   sequents over `Notation`, with `Display` as one of its tables; `Rule`
   as a classical rule and a position, with its tables in one place and
   a file of its own.
6. **`proofs/interactive.rs` along its seams** (state and undo, rule
   validation, reading a state back, the translation to terms, the
   search), and whether a shared forest is worth it so that `Reading`
   and `Derivation` are owned values.
7. **The lint allowances** of `core/src/lib.rs` removed: the dead items
   deleted, the feature-only ones gated.
8. **The invariants of section 1.3** turned into a type, an assertion or
   a test where one sentence of code does it, and the tests of 1.4 that
   are missing for a stated behaviour or in excess.
9. **The rules files**: one per module path, each loading with its
   module, so that reading a file of the exports does not load the
   focused engine's six hundred lines; `.claude/rules/claude-infra.md`
   and CLAUDE.md follow.

## Constraints

- The command's behaviour, every JSON form and every snapshot stay as
  they are. A change of behaviour that the work finds necessary is
  reported and committed on its own.
- `search/focus/` and `search/net.rs` change only at their public
  signatures; the target set's counters stay identical, and pinned CPU
  time on its rows over a second stays within two percent.
- Dependencies only where they earn their place (D10).
- This step does not add quantifiers. It leaves the place for them, and
  the note says what the first step of adding them is.

## Verification

Every check after every commit that touches code: clippy, the tests,
both `cargo hack` runs; `bench/targets.sh` at the end and after any
commit that touches `search/`; `nix flake check`; `nix build .#doc`,
and the rustdoc front page read as a stranger would read it.

## Deliverables

- Thematic jj commits, each of which builds and passes alone.
- `plan/notes/api.md`.
- `plan/reports/23-api.md`: the surface before and after, what was
  renamed or removed (a table a later session can search), the
  first-order plan in a page, decisions, deviations, open questions,
  what steps 24 to 29 must know.
