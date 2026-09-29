# Step 12 report: proof certificates for Rocq

Session of 2026-09-29, from `plan/12-certificates.md`. Realises the `rocq`
feature of D14 and applies D15 to it from the start.

## Outcome

`linlog::export::rocq`, behind the default cargo feature `rocq`, writes a
finished derivation as a Rocq proof script for NanoYalla 1.1.3, the kernel
of Click & coLLecT: a `Lemma` whose statement is the one-sided sequent as
a list of the kernel's formulas over the atoms as `formula` binders, and
whose proof applies one derived rule of the kernel per inference,
conclusion first, closed by `Qed`. `linlog prove --format rocq` and
`linlog check --format rocq` print it after the verdict as a `(* … *)`
comment, `--standalone` adds the import line. The flake check `rocq`
builds the kernel from a pinned flake input with nixpkgs' Rocq 9.1.1 and
compiles every certificate the core tests pin plus two the CLI writes,
requiring Rocq to print nothing; it passes, and a certificate with an
`admit` or with a wrong formula fails it (below). Certified: every
classical fragment, MLL to full LL, with contraction, weakening,
dereliction and promotion; intuitionistic proofs as the classical proofs
they are; refused with a message, before anything is written: a proof
with Mix, one with the weakening of affine mode, and a proof in progress.
Three commits: "Export derivations as Rocq scripts for NanoYalla", "Check
the Rocq certificates in nix", "Document the Rocq certificates".

## The kernel and why

The live sources were read this session (Click & coLLecT at its `master`
head `6b1c25ff30a1c1dbfccdc0c1cebc272a67ab0137` of 2025-04-07, whose
`nanoyalla/` last changed in commit `2fc940cd` of 2021-06-02; Yalla at
tag v2.0.7 of 2025-03-26 and its unreleased `master`); the facts below
come from them, not from the notes.

**NanoYalla** (`nanoyalla/` in the Click & coLLecT repository, LGPL-2.1,
version 1.1.3 by its README) is four files: `nanoll.v`, the trusted base,
an inductive `ll : list formula -> Type` with `ax_r`, a transposition
`ex_t_r`, and the fifteen logical and exponential rules, no Mix and no
cut, over `formula` with `Atom := nat`; `macroll.v`, the derived rules:
`<rule>_r_ext l1 …` acting on a formula at any position of the list, an
`ex_perm_r p l` exchange by a permutation given as a list of indices,
`ax_r_ext` (an axiom on any formula) and the tactics `ax_expansion` and
`cbn_sequent`; and two variants of cut (`axiomcut.v` postulates it,
`yallacut.v` proves it through Yalla), which our cut-free certificates do
not import. It builds standalone with Rocq 9.1.1 and `rocq-stdlib` in
half a second (its `From Coq Require Import Lia` draws a deprecation
warning, nothing else), and needs no Yalla.

**Yalla's own kernels.** At v2.0.7 `microyalla/` holds `ll.v` and `ill.v`
with an inline `Permutation_Type` inductive as the exchange rule; on
`master` (the unreleased 2.1.0, "tested with Rocq 9.2", needing OLlibs
2.1.1) `microyalla/nanoll.v` is byte for byte Click & coLLecT's `nanoll.v`
and `microll.v` has an adjacent-swap exchange, and neither has the
`_ext` layer. Full Yalla needs OLlibs (vendored at v2.0.7, external on
`master`) and a specific Rocq minor version, and nixpkgs packages neither
Yalla nor OLlibs (`pkgs/development/rocq-modules` has no such entry);
its `ll` is parameterised by a `pfrag` record with `pmix0`/`pmix2`.

The choice is NanoYalla, for three reasons: it is the only kernel with a
derived-rule layer that acts at a position and a permutation lemma, so a
script needs no reasoning about list equalities; it builds with the Rocq
that nixpkgs ships, with nothing but the standard library; and it is what
Click & coLLecT's users already have installed. The version pinned is
NanoYalla 1.1.3 as the flake input `nanoyalla`, the Click & coLLecT
repository at commit `6b1c25ff…` with `flake = false` (the directory has
no release of its own; `nix flake update` keeps a pinned revision); the
constant `export::rocq::NANOYALLA` names the version, as `typst::CURRYST`
does for curryst. The LGPL kernel is not vendored: linlog's tree holds no
line of it, the check fetches it, and a user installs it from the Click &
coLLecT repository (`./configure && make && make install`, or
`rocq compile -R . NanoYalla` on `nanoll.v` and `macroll.v`), which the
README and the `--help` of `--format rocq` say.

**Rocq.** The pinned nixpkgs has `rocq-core` 9.1.1 as its default
(`rocqPackages_9_1`; 9.0 to 9.3 exist) and the shim `coq` 9.1.1 on top of
it, plus `rocqPackages.stdlib`. The check uses `rocq compile` and sets
`ROCQPATH` to the standard library, since the packages carry no setup
hook.

## The API and the options

- `rocq::derivation(&derivation, form, &options) -> Result<String,
  Unsupported>`: the lemma alone (`Form::Fragment`) or the file
  (`Form::Standalone`, the prelude, a blank line, the lemma). `Form` is
  now gated on `latex`, `typst` or `rocq`, and `notation` (for `walk`) on
  those and `svg`.
- `rocq::Options { lemma: String, prelude: String }`, `Default` (`lemma`
  `"certificate"`, `prelude` `"From NanoYalla Require Import macroll."`),
  `Clone`, `PartialEq`, `Hash`, serde behind `serialize`. D15: these are
  the two things a user might vary. A kernel flavour is not a field
  because one kernel is supported; the verdict comment is the CLI's
  (`note`, shared by every source format), not the library's, and whether
  it is written belongs to the CLI-level output options 15h designs. How
  each front end sets them: the CLI uses the defaults today and gets
  `--lemma NAME` and `--prelude LINE` (or a config file) with 15h's
  `--style` flags; the web front end holds the `Options` as JSON in its
  settings (`{"lemma": "…", "prelude": "…"}`) and passes them to a wasm
  export of `derivation`; any other wrapper constructs the value.
- `rocq::Unsupported { Open, Mix, AffineWeakening }`, a small `Error`
  enum of its own, with messages that name the kernel and the rule, which
  the CLI prefixes with "no certificate:" at exit status 2.
- `rocq::NANOYALLA`, the version string.
- The CLI: `Format::Rocq` (its `--help` names the kernel, the version, the
  repository and Rocq 9), an arm in `derivation`, `(* … *)` in `note`,
  `--standalone` accepted by `form` for `rocq` and refused for the
  one-form formats with a message that now lists three formats.

The formula printer is the kernel's constructors, prefix and fully
bracketed (`tens (parr A B) (dual C)`), not the `LLNotations` module:
its infix operators are all at one level without an associativity a
generated text should depend on, and `0`/`1` as formula notations would
clash with the index lists of `ex_perm_r`. Atoms are `formula` binders of
the lemma (`Lemma certificate (A B : formula) : ll […]`), as Click &
coLLecT's `Variable`s are, so the lemma holds for any formulas
substituted for the atoms and the axiom is `ax_expansion` on a pair
`dual A`, `A` in either order. A name becomes an identifier by keeping
letters, digits, `_` and `'`, writing any other character as `_<hex>_`,
prefixing `_` to a leading digit or `'`, and appending `'` until it
clashes with none of Rocq's keywords, the kernel names a script mentions
(`RESERVED`), the lemma's name, or another atom; `tens_r_ext` as an atom
comes out as `tens_r_ext'`.

## The permutation strategy

The kernel's sequents are lists and ours are multisets in ascending
occurrence order. The exporter keeps, for every inference, the exact list
Rocq will show as its goal (`Script::goals`, filled in when the
conclusion's tactic is written; the root's list is the sequent in id
order, which the lemma states). Every `_ext` lemma takes the list `l1` of
the formulas before the principal one and lets unification find the rest,
so every rule but `⊗` acts in place: `parr_r_ext l1` leaves `A :: B` where
`A ⅋ B` was, `co_r_ext l1` leaves the two copies adjacent, `wk_r_ext l1`
removes the formula, `oc_r_ext l1 (A) l2` gets both contexts without their
`?` (the only rule that needs `l2` spelled out, since `map wn l2` is not
invertible by unification), `bot_r_ext`, `de_r_ext`, `plus_r1_ext`,
`plus_r2_ext`, `with_r_ext` likewise, `top_r_ext l1` and `one_r_ext` close
a goal. `tens_r_ext l1 A B l2 : ll (l1 ++ A :: nil) -> ll (B :: l2) -> ll
(l1 ++ tens A B :: l2)` is the one rule that constrains the order: the
left premise's context must stand before the `⊗` and the right one's
after it. So the exporter partitions the current list by membership in
the left premise's sequent (stably, consuming one copy per match, the
first `⊗` occurrence being the principal), forms the target `Γ_left, A ⊗
B, Γ_right`, and when the current list differs emits one
`apply (ex_perm_r p target).` first, where `p[i]` is the position in the
target of the current list's `i`-th formula (verified against the
kernel's `transpL (permL_of_perm p) l`, which yields the list whose
position `i` holds `l[p[i]]`; equal ids are equal formulas, so the first
unused match serves). `A * B |- B * A` shows it: `parr_r_ext []`, then
`ex_perm_r [2; 0; 1] [dual B; tens B A; dual A]`, then `tens_r_ext [dual
B]`. Nothing else is permuted; a certificate has at most one exchange per
`⊗`. The two premises of a binary rule are written in `{ … }` blocks,
indented two spaces per level, in the walk's enter order (`notation::walk`
handles the explicit stack; enters are the prefix order a tactic script
wants, unlike ebproof's exits).

## What is certified

| fragment or mode | certificate | notes |
|---|---|---|
| MLL, MLL with units, MALL, MELL, LL, classical | yes | every rule maps to a `_ext` lemma; `?d`/`?c`/`?w`/`!` to `de`/`co`/`wk`/`oc` |
| classical with Mix, proof without a Mix inference | yes | the derivation decides, not the mode |
| classical with Mix, proof with Mix | refused, `Unsupported::Mix` | `nanoll.v` has no Mix; Yalla's full `ll` would (`pmix2`) |
| affine, proof without `wk` | yes | |
| affine, proof with `wk` | refused, `Unsupported::AffineWeakening` | the kernel weakens `?` formulas only |
| intuitionistic (any fragment) | yes, as the classical derivation | `Rule::classical` on the two-sided view; the sequent certified is the one-sided one, `ll [dual A; tens A (dual B); B]` for `A, A ⊸ B ⊢ B` |
| a proof in progress | refused, `Unsupported::Open` | no `Admitted`, ever |

The intuitionistic decision: what the term proves is a classical proof of
the lowered sequent, and NanoYalla checks exactly that; the ILL reading's
ambiguity for `⊤`/`0`-built succedents (step 8) does not reach the kernel,
since the one-sided sequent is the same whichever root the reading calls
the goal. A certificate in Yalla's `ill` would state the two-sided sequent
the reading prints and would carry that ambiguity into the statement; it
is a follow-up (below), not built, because it needs full Yalla with OLlibs
(no nixpkgs package, a Rocq minor version pinned per release, a build of
minutes) and a script over `ill`'s primitive rules with `Permutation_Type`
proofs for every exchange, since no `_ext` layer exists for it.

## The check in nix

`modules/rocq.nix` adds `checks.rocq`: a `runCommand` with the CLI,
`rocq-core` and `rocqPackages.stdlib`, `ROCQPATH` set to the standard
library, that copies `nanoyalla/` from the input, compiles `nanoll.v` and
`macroll.v` with `-R nanoyalla NanoYalla` (its warnings go to a log shown
only on failure), copies the `.v` snapshots in, writes `cli.v` (`A * top
|- A * (B + top)`: `⊤`, `⊕₂`, a `⊗` needing no exchange) and `cli_ill.v`
(`!A, A -o B |- B * !A` in intuitionistic mode), and compiles each with
`rocq compile … 2>&1 | tee log`, failing on a non-empty log. Rocq is
silent on success, so the empty log is the evidence; a file name must be
a Rocq identifier (`cli-ill.v` was refused, hence the underscore). The
check is part of `nix flake check`. The cost is the closure of
`rocq-core` 9.1.1, 1.19 GB plus 78 MB for the standard library, all from
`cache.nixos.org`; the kernel and the certificates compile in about a
second. That is heavy next to the export check's TeX Live and Typst, but
a download, not a build, and keeping the certificates in the one command
CI and the author run is worth a minute of CI time; `nix build
.#checks.x86_64-linux.rocq` runs it alone, and if CI time becomes a
problem the check moves to its own job without a code change.

`workspace.nix` already keeps `core/tests/snapshots` in the crane source,
so the `.v` files ride along; `deny.toml` is untouched (no crate
dependency was added).

## Lean and Agda

Assessed from the notes gathered on 2026-09-29 (`plan/notes/export-targets.md`);
neither repository was re-fetched this session, so the gaps named may
have moved.

- **Lean 4**: FormalizedFormalLogic/LinearLogic (Apache-2.0) defines
  one-sided calculi over `Multiset Formula`, which would spare the
  exchange bookkeeping entirely (a multiset sequent has no order), but
  the fetched excerpt has no units, so MLL with units, MALL's `⊤`/`0` and
  everything above would have no target; cut elimination is in an open
  pull request and the project calls itself early-stage; nothing is in
  Mathlib. A target would need: their inductive's exact constructors
  (names, the shape of the `⊗` rule's context split, how `!` demands a
  `?` context over a multiset), a formula printer for their syntax, a
  `theorem … : Derivation {…} := by` script or, more naturally for a
  multiset calculus, a proof term built by constructor application, and a
  `lake` project pinned to a Lean toolchain and Mathlib revision for the
  check, whose closure is large. Worth doing once the project has units
  and a release; the exporter's structure (a walk emitting one step per
  inference from tracked goals) carries over with the goal tracking
  removed.
- **Agda**: no maintained kernel for classical linear logic exists to
  target (ncill is non-commutative intuitionistic, the rest is course
  material). Not worth a target.

## Decisions where the prompt left room

- **`macroll`, not `macrollcut`.** Click & coLLecT imports the cut
  variant because its proofs may have cuts; ours never do, so the
  certificate's trusted base is `nanoll.v` alone: no axiom (`axiomcut.v`
  postulates one) and no Yalla.
- **A `Lemma` with binders, not `Goal` in a `Section`.** Click & coLLecT
  writes `Section TheProof. Variable a b : formula. Goal ll […]. … Qed.
  End TheProof.`; a named lemma with `(A B : formula)` binders is one
  block that pastes into a development and can be referred to, and it is
  where the D15 option `lemma` lives.
- **Constructors, not notations**, for the reasons in "The API".
- **No exchange except before a `⊗`**, and goal tracking instead of
  normalising every goal to id order: fewer tactics and a script that
  reads like the derivation.
- **Refuse rather than approximate.** Mix could be simulated by nothing
  in NanoYalla; affine weakening of a non-`?` formula has no rule; an
  open goal must not become `Admitted`. All three are `Unsupported`
  before any text is written, so a partial script never leaks.
- **`Unsupported` is its own error type**, not variants of
  `linlog::Error`: it is the export's boundary, as `NetError` is the
  nets', and the CLI turns it into exit 2 through `anyhow`.
- **The check stays in `nix flake check`** (above).
- **Snapshots**: `pin_proof` now also pins `name.v`, so `mll`, `mall`,
  `mell` and `ill` get certificates next to their trees, and `ll.v`
  (`!(A & B) |- !A * !B`, a contraction below a `⊗`, promotions with
  contexts on both sides) is pinned by the `certificates` test, which
  also checks the options and the two refusals; `open_goal` checks the
  third. The CLI test `rocq_format` pins the fragment for `A |- A`, the
  standalone form through `check`, and the Mix refusal.
- **`interact` has no `show rocq`.** `show latex|typst|svg` draw the
  user's partial derivation, which has no certificate; a finished
  session's certificate is `Interactive::proof()` followed by
  `proof.derivation()`, the term's view, and belongs with a `proof`
  command the session does not have yet. Follow-up.

## Deviations from the prompt, with reasons

- **No Yalla `ill` target** for intuitionistic derivations: the prompt
  asked for it "if Yalla's ILL development makes it reasonable", and it
  does not yet (above). Intuitionistic proofs are certified classically,
  and the report, the README and `--help` say so.
- **No `--lemma` flag.** The option exists in the library as D15 asks;
  the CLI flag joins 15h's output flags rather than adding one flag for
  one format now.
- **The Lean and Agda assessment was not re-verified against the live
  repositories** this session; it rests on the notes of the same day.

## Open questions and follow-ups

- **Yalla `ill` certificates** (15 material): a second kernel behind the
  same `Options` (a `kernel` field then), full Yalla and OLlibs as flake
  inputs built at their Rocq version, the two-sided statement from the
  reading, `Permutation_Type` witnesses for the exchanges, and the ILL
  ambiguity of step 8 stated in the report of that step.
- **Mix through full Yalla**: the same kernel would take `mix2_r`; not
  worth a second kernel on its own.
- **`--lemma`, `--prelude`** in the CLI and the verdict comment as an
  option: 15h.
- **`show rocq` / a `proof` command in `interact`** for a finished
  session's certificate through the term.
- **A Lean target** once FormalizedFormalLogic/LinearLogic has units.
- **Identifier escaping** writes non-ASCII characters as code points;
  Rocq accepts many Unicode letters in identifiers, so `α` could stay
  `α`. The escape is safe and reversible enough; refining it needs Rocq's
  exact identifier grammar.
- **Large proofs**: `ex_perm_r` makes Rocq compute `permL_of_perm`, whose
  cost grows with the sequent's width; nothing was measured beyond the
  test sequents. A width where `apply` becomes slow would want the
  exchange written as a chain of `ex_t_r` swaps instead.

## For step 13 and the web front end

- The web front end certifies a finished proof with
  `rocq::derivation(&proof.derivation()?, Form::Standalone, &options)`
  and offers the text as a `.v` download; `options` is the JSON it holds.
  In intuitionistic mode it passes `proof.derivation()` or
  `proof.two_sided_derivation()` alike (the output is the same).
- Nothing here touches the search; step 13's parallel engine returns the
  same `Proof` type, whose derivation the exporter reads.

## Verification

- `cargo clippy --workspace --all-targets -- --deny warnings`: clean.
- `cargo test --workspace`: all suites pass (core lib 106 passed and 6
  ignored, core integration suites 10, 10, 17, 2, 6 and 9 passed, CLI
  suites pass; the `export` suite is 6 tests, `certificates` and the
  extended `open_goal` among them).
- `cargo hack check --each-feature -p linlog` and `cargo hack check
  --feature-powerset --depth 2 -p linlog`: pass. The first run of the
  flake's `features` check caught `Form` still gated on `latex`/`typst`
  only, which broke `--features rocq` alone; fixed and folded into the
  exporter's commit.
- Every generated certificate compiled locally with Rocq 9.1.1 and the
  standard library against the kernel built from the pinned commit: the
  five snapshots, and `top.v`, `units.v` (`|- bot, 1`), `copies.v`
  (`?(A * A), ~A, ~A, ~A, ~A |- 0 & top`, two derelictions of a
  contracted `?`, a `⊗` occurrence repeated in one sequent) and `names.v`
  (atoms `tens_r_ext`, `x_1`, `foo`), each with no output.
- Negative checks of the check: `mll.v` with `ax_expansion.` replaced by
  `admit.` fails at `Qed` with

  ```
  Error:  (in proof certificate): Attempt to save an incomplete proof
  (the proof term is not complete because of given up (admitted) goals).
  ```

  and with `tens B A` replaced by `tens A B` in the statement fails at the
  first tactic with `Error: Tactic failure: Cannot solve this goal.`
- `nix build .#checks.x86_64-linux.rocq`: builds; `nix log` of it is
  empty, which is what the check requires (the kernel's warnings are
  captured, every certificate printed nothing).
- `nix flake check` after `jj st`: passes (build, clippy, test, doc,
  deny, features, export, rocq, deadnix, actionlint, treefmt,
  claude-hooks).
