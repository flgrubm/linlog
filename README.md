# linlog

A linear logic suite for all your needs: a command line program and a Rust
library that parse, print and decide sequents of classical and
intuitionistic linear logic and their fragments, keep the proofs in a
checkable form and show them as derivation trees or proof nets.

[API documentation](https://flgrubm.github.io/linlog/) (rustdoc of `main`,
rebuilt on every push).

## Usage

The command line program is `linlog` (`nix run github:flgrubm/linlog -- …`,
`nix build`, or `cargo run -p linlog-cli -- …` in a checkout). `linlog --help`
and `linlog <command> --help` document every option.

`linlog prove` decides a sequent. The first line gives the verdict, the
fragment the sequent was detected to live in and the engine that searched;
a proof follows as a derivation tree of the one-sided sequent calculus:

```console
$ linlog prove "A, A -o B |- B"
provable (MLL, classical, net engine)
─────── ax   ─────── ax
⊢ ~A, A      ⊢ ~B, B
──────────────────── ⊗
  ⊢ ~A, A ⊗ ~B, B

$ linlog prove "A & B |- A + B"
provable (ALL, classical, focus engine)
    ─────── ax
    ⊢ ~A, A
  ─────────── ⊕₁
  ⊢ ~A, A ⊕ B
──────────────── ⊕₁
⊢ ~A ⊕ ~B, A ⊕ B

$ linlog prove "|- A par B, ~A, ~B"
unprovable (MLL, classical, net engine): the search was exhaustive
```

Three engines serve classical logic: for MLL without units whose literals
occur at most twice each, the *net engine* searches for an axiom linking
that makes the sequent's formula trees a proof net; for two additive-only
formulas the *additive engine* recurses on pairs of subformulas; and for
everything else the *focus engine* runs a focused sequent search over
bitsets (on repeated literals its count-based pruning beats the linking
search by orders of magnitude). `--mix`, `--affine` and `--intuitionistic`
choose the logic, `--fragment` and `--engine focus|net|two-sided|additive`
override what detection picks, `--timeout 10s`
and `--copies N` bound the search, `--quiet` prints the verdict line only
and `--stats` what the search cost, in the counters of the engine that
ran:

```console
$ linlog prove --mix --stats "|- A par B, ~A, ~B"
provable (MLL, classical with Mix, net engine)
─────── ax   ─────── ax
⊢ A, ~A      ⊢ B, ~B
──────────────────── mix
   ⊢ A, B, ~A, ~B
   ─────────────── ⅋
   ⊢ A ⅋ B, ~A, ~B
literals chosen: 2
links tried: 2
exact tests run: 2
time: 63.45µs
$ linlog prove --engine focus --stats --quiet "|- A * B, C * (~A par ~B), ~C"
provable (MLL, classical, focus engine)
stable sequents visited: 2 (0 from the memo)
memo entries at most: 2
splits examined: 3
time: 56.66µs
```

`--format net` shows the proof as a proof net instead of a derivation: the
sequent, the axiom links as pairs of literals with their positions in the
sequent's subformula numbering, and the verdict of the correctness
criterion. With the net engine this is the net the search found; with the
focus engine it is read off the proof. Proof nets exist for MLL without
units, with or without Mix:

```console
$ linlog prove --format net "|- A * B, C * (~A par ~B), ~C"
provable (MLL, classical, net engine)
⊢ A ⊗ B, C ⊗ (~A ⅋ ~B), ~C
A[1] — ~A[6]
B[2] — ~B[7]
C[4] — ~C[8]
proof net
$ linlog prove --engine net "A & B |- A"
error: proof nets exist for MLL without units only, not for ALL
```

With exponentials (MELL and full LL) the focus engine searches dyadic
sequents, copying a `?` formula at most `--copies` times on any branch
(3 by default) and deepening that bound from zero. The derivation shows the
standard rules: dereliction, contraction, weakening and promotion.

```console
$ linlog prove "!A |- A * A"
provable (MELL, classical, focus engine)
─────── ax    ─────── ax
⊢ ~A, A       ⊢ ~A, A
──────── ?d   ──────── ?d
⊢ ?~A, A      ⊢ ?~A, A
────────────────────── ⊗
  ⊢ ?~A, ?~A, A ⊗ A
  ───────────────── ?c
    ⊢ ?~A, A ⊗ A
$ linlog prove "!A, !(A -o B), !(B -o C) |- C"
provable (MELL, classical, focus engine)
              ─────── ax   ─────── ax
              ⊢ ~B, B      ⊢ ~C, C
─────── ax    ──────────────────── ⊗
⊢ ~A, A         ⊢ ~B, B ⊗ ~C, C
──────── ?d    ────────────────── ?d
⊢ ?~A, A       ⊢ ~B, ?(B ⊗ ~C), C
───────────────────────────────── ⊗
   ⊢ ?~A, A ⊗ ~B, ?(B ⊗ ~C), C
  ────────────────────────────── ?d
  ⊢ ?~A, ?(A ⊗ ~B), ?(B ⊗ ~C), C
```

Provability in MELL has no known decision procedure, so the verdict is
three-valued: "unprovable" is reported only when a bound was searched
exhaustively without ever hitting it, and "unknown" when every bound up to
`--copies` was hit:

```console
$ linlog prove -q --copies 1 "!A, !(A -o B), !(B -o C) |- C"
unknown (MELL, classical, focus engine): the copy bound of 1 was reached; raise it with --copies
$ linlog prove -q "A |- !A"
unprovable (MELL, classical, focus engine): the search was exhaustive
$ linlog prove -q "!(A -o A * A), A |- ?B"
unknown (MELL, classical, focus engine): the copy bound of 3 was reached; raise it with --copies
```

`--affine` allows weakening: a hypothesis may go unused, which the
derivation shows as `wk` below the leaf that leaves it over. With
exponentials the affine search is bounded by `--copies` like the linear
one:

```console
$ linlog prove -a "A, B |- A"
provable (MLL, classical affine, focus engine)
  ─────── ax
  ⊢ ~A, A
─────────── wk
⊢ ~A, ~B, A
$ linlog prove -q -a "!(A -o A * A), A |- ?B"
unknown (MELL, classical affine, focus engine): the copy bound of 3 was reached; raise it with --copies
```

`--intuitionistic` (`-i`) reads the sequent as intuitionistic linear logic:
one formula on the right of `⊢` and pars only as implications, which the
one-sided form keeps as `~A ⅋ B`. The verdict line names the intuitionistic
fragment, and the derivation is two-sided with the rules of ILL. IMLL
without units goes to the net engine (the classical net of the one-sided
sequent is always an intuitionistic proof), a sequent of two additive-only
formulas to the *additive engine* (a recursion on pairs of subformulas,
linear in the product of their sizes, in every mode), and everything else
to the *two-sided engine*, the focused search keeping one goal on every
branch, with the same copy bound and affine mode as the classical one:

```console
$ linlog prove -i "A -o B -o C, A * B |- C"
provable (IMLL, intuitionistic, net engine)
           ───── ax   ───── ax
           B ⊢ B      C ⊢ C
───── ax   ──────────────── ⊸L
A ⊢ A        B ⊸ C, B ⊢ C
───────────────────────── ⊸L
  A ⊸ (B ⊸ C), A, B ⊢ C
  ────────────────────── ⊗L
  A ⊸ (B ⊸ C), A ⊗ B ⊢ C
$ linlog prove -i "!A, !(A -o B) |- !B & A"
provable (ILL, intuitionistic, two-sided engine)
───── ax
A ⊢ A
────── !L   ───── ax
!A ⊢ A      B ⊢ B
───────────────── ⊸L         ───── ax
  !A, A ⊸ B ⊢ B              A ⊢ A
 ──────────────── !L         ────── !L
 !A, !(A ⊸ B) ⊢ B            !A ⊢ A
 ───────────────── !R   ──────────────── !w
 !A, !(A ⊸ B) ⊢ !B      !A, !(A ⊸ B) ⊢ A
 ─────────────────────────────────────── &R
          !A, !(A ⊸ B) ⊢ !B & A
$ linlog prove -i -q --stats "(A & B) + (A & C) |- A & (B + C)"
provable (IALL, intuitionistic, additive engine)
pairs of subformulas visited: 20 (0 from the memo)
memo entries: 20
time: 89.90µs
$ linlog prove -i "|- A par B"
error: not an intuitionistic sequent: the subformula A ⅋ B is neither an intuitionistic formula nor the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)
```

Classical linear logic proves more than intuitionistic linear logic once
`0` is around, and the two-sided search knows the difference:

```console
$ linlog prove -q "((A * top) & (B * top)) -o 0 |- (A -o C) + (B -o C)"
provable (MALL, classical, focus engine)
$ linlog prove -i -q "((A * top) & (B * top)) -o 0 |- (A -o C) + (B -o C)"
unprovable (IMALL, intuitionistic, two-sided engine): the search was exhaustive
```

The exit status tells scripts the verdict: 0 provable, 1 unprovable, 3
unknown (the copy bound, the time limit or Ctrl-C stopped the search), 2 an
error, such as a sequent outside the asserted fragment or an engine forced
on a sequent it cannot search.

`--format json` writes the outcome as one JSON object, which is also a proof
file that `linlog check` verifies independently of the search (pass the same
logic flags):

```console
$ linlog prove --format json "A |- A"
{"verdict":"proved","fragment":"MLL","mode":{"intuitionistic":false,"affine":false,"mix":false},"engine":"net","statistics":{"nodes":1,"memo_hits":0,"memo_entries":0,"splits":0,"links":1,"tests":1},"sequent":{"terms":[{"D":0},{"V":0}],"ids":[0,1],"var_dict":["A"]},"proof":[{"ax":[0,1]}]}
$ linlog prove --format json "A |- A" | linlog check --quiet
valid proof of ⊢ ~A, A (classical)
```

`linlog interact` proves a sequent step by step, reading commands from
standard input: the open goals are listed with the position of every
formula, `rules` names the rules that act on a formula, `apply` applies one
(a `⊗` or Mix takes the positions of the formulas that go to its left
premise), `undo` retracts the last step, `close` lets the search close one
goal or all of them, `show` draws the derivation so far with the open
goals as bare sequents (`show latex` and `show typst` as proof trees), `save` and `load` keep a session as JSON, and
`proof` checks the finished proof independently and prints it or writes
it for `check`. In intuitionistic mode the goals are two-sided and the
rules carry the names of ILL:

```console
$ linlog interact -i "A, A -o B |- B"
> goals
goal 0: 0: A, 1: A ⊸ B ⊢ 2: B
> rules 0 1
⊸L (with a split)
> apply 0 1 -oL 2
error: a premise would have 2 formulas on the right of ⊢ instead of one
> apply 0 1 -oL 0
opened goal 1: 0: A ⊢ 1: A
opened goal 2: 0: B ⊢ 1: B
> apply 1 0 ax
closed
> show
───── ax
A ⊢ A      B ⊢ B
──────────────── ⊸L
  A, A ⊸ B ⊢ B
> close
goal 2: proved (IMLL, intuitionistic, two-sided engine)
no goal is open: `proof` checks the proof
> proof
valid proof (intuitionistic)
───── ax   ───── ax
A ⊢ A      B ⊢ B
──────────────── ⊸L
  A, A ⊸ B ⊢ B
```

The exit status is 0 when the session ends with a finished proof that
checks, 1 otherwise. The same operations are the library's `Interactive`
type, whose JSON form is what a web client will hold between requests.

`linlog seq` prints a sequent one-sided in negation normal form, or
two-sided as intuitionistic linear logic reads it, converts it to JSON
(which `--json-input` reads back), or names its fragment:

```console
$ linlog seq print "A * B -o C |- ~C -o ~(A * B)"
⊢ (A ⊗ B) ⊗ ~C, C ⅋ (~A ⅋ ~B)
$ linlog seq print -i "A * B -o C |- ~C -o ~(A * B)"
(A ⊗ B) ⊸ C ⊢ (A ⊗ B) ⊸ C
$ linlog seq fragment -i "A & B |- 1"
IMALL
$ linlog seq json "A |- A"
{"terms":[{"D":0},{"V":0}],"ids":[0,1],"var_dict":["A"]}
$ linlog seq fragment "A & B |- 1"
MALL
```

`--format latex` and `--format typst` write the derivation of `prove` and
`check` as a proof tree to paste into a paper or slides: LaTeX for the
[ebproof](https://ctan.org/pkg/ebproof) package with the connectives of
[cmll](https://ctan.org/pkg/cmll) and amssymb (turnstiles aligned in
two-sided trees), Typst for the
[curryst](https://typst.app/universe/package/curryst) package. The verdict
line becomes a comment. `seq print` takes the same formats for a sequent:

```console
$ linlog prove -i --format latex "A, A -o B |- B"
% provable (IMLL, intuitionistic, net engine)
\begin{prooftree}
\infer0[$\mathrm{ax}$]{A &\vdash A}
\infer0[$\mathrm{ax}$]{B &\vdash B}
\infer2[$\multimap\mathrm{L}$]{A, A \multimap B &\vdash B}
\end{prooftree}
$ linlog prove --format typst "A & B |- A + B"
// provable (ALL, classical, additive engine)
#prooftree(
  rule(
    name: $⊕_1$,
    rule(
      name: $⊕_1$,
      rule(name: $"ax"$, $⊢ A^⊥, A$),
      $⊢ A^⊥, A ⊕ B$,
    ),
    $⊢ A^⊥ ⊕ B^⊥, A ⊕ B$,
  ),
)
$ linlog seq print -i --format latex "A * B -o C |- ~C -o ~(A * B)"
$(A \otimes B) \multimap C \vdash (A \otimes B) \multimap C$
$ linlog seq print --format typst "A * B -o C |- ~C -o ~(A * B)"
$⊢ (A ⊗ B) ⊗ C^⊥, C ⅋ (A^⊥ ⅋ B^⊥)$
```

`--standalone` writes a document that compiles on its own instead, cropped
to the tree: `pdflatex` needs the ebproof, cmll, amsfonts and standalone
packages, `typst compile` fetches curryst 0.6.0 on first use. Typst
refuses a curryst tree more than about eleven inferences high; the LaTeX
tree has no such limit.

```console
$ linlog prove --format latex --standalone --output proof.tex "!A |- A * !A"
$ pdflatex proof.tex
```

Names of more than one letter are set in italics (`\mathit{foo}`,
`italic("foo")`), with the characters LaTeX or Typst treat specially
escaped. In `interact`, `show latex` and `show typst` draw the derivation
so far, an open goal as its sequent under vertical dots.

The syntax: `*`/`⊗` tensor, `|`/`par`/`⅋` par, `&` with, `+`/`⊕` plus,
`-o`/`⊸` linear implication, `~A` or `A^` negation, `!` and `?`, and the
units `1`, `bot`/`⊥`, `top`/`⊤`, `0`; `|-` or `⊢` separates the sides.

## What exists and what is planned

Built:

- Parsing and printing of sequents in the syntax above, with the ASCII and
  Unicode spellings of every connective, and a compact JSON form.
- Detection of the fragment a sequent lives in (MLL, MLL with units, ALL,
  MALL, MELL, LL, and their intuitionistic counterparts IMLL to ILL) and
  the modes classical, affine, intuitionistic and Mix as user choices.
- Intuitionistic linear logic on the same one-sided representation: a
  sequent is read two-sided by the polarization of its subformulas (one
  goal, hypotheses, `⊸` recovered from `~A ⅋ B`), printed as `Γ ⊢ A`, and
  proved by the two-sided focused search, by the embedding of IMLL into
  MLL proof nets, or by the additive fast path.
- Proofs as compact terms over subformula occurrences, an independent
  checker that decides whether a term proves its sequent (in
  intuitionistic mode also that every sequent of the proof has one goal),
  and a derivation view that unfolds a term into the tree of the standard
  sequent calculus, one-sided or two-sided with the rules of ILL, printed
  as text.
- Automatic proof search for every fragment, MLL to full LL and IMLL to
  ILL, with or without Mix, returning a checked proof, "unprovable" after
  an exhaustive search, or "unknown" with the reason: a focused sequent
  engine over dyadic sequents of occurrence bitsets with a memo,
  count-based pruning, a per-branch bound on the copies of `?` formulas
  that deepens iteratively, and a loop check, one-sided or two-sided; for
  MLL without units a proof-net engine that searches the axiom linkings
  with count checks, constant-time cycle rejections, the exact acyclicity
  test and a symmetry break for repeated literal conclusions, then
  sequentializes the net it finds; and for two additive-only formulas a
  recursion on subformula pairs.
- Affine mode, where weakening is allowed, in every fragment.
- Proof nets for MLL, with or without Mix, as a representation of their
  own: proof structures over the subformula occurrences, an independent
  correctness criterion (Danos–Regnier, decided by Yeo's deletion test on
  the coloured structure graph, with a switching cycle or the
  disconnection named when it fails), sequentialization into a checked
  proof and desequentialization of a proof into its net, a text form and
  a JSON form.
- Interactive proving: a proof in progress as a derivation with open
  goals, rules applied to a formula of a goal and validated (the
  connective, the mode, the context a promotion or an axiom needs, one
  goal per premise in intuitionistic mode), undo, the search closing any
  goal, translation of the finished derivation into a term the checker
  validates, and a JSON form of the session.
- Export of sequents and derivations, finished or in progress, to LaTeX
  (ebproof proof trees) and Typst (curryst proof trees), as fragments or
  standalone documents.
- The `linlog` command: `prove`, `check`, `interact` and `seq`, with time
  limits, Ctrl-C, statistics, JSON output, proof nets, and LaTeX and Typst
  output.

Planned, in roughly this order:

- Drawings of derivations and proof nets as SVG, and proof certificates
  for Rocq.
- Parallel search, a benchmark harness with the standard problem
  libraries, and performance work driven by its numbers.
- Later: proof nets with exponential boxes, essential nets for
  intuitionistic MLL, the inverse method, the Lambek calculus, and a web
  front end.

The design follows [Click and Collect](https://www.click-and-collect.linear-logic.org)
where it is good and departs from it where it is not. Feature requests and
contributions are welcome. All code is licensed under the EUPL.

## Architecture

Two crates, with a third to come: the library `linlog` in `core/` holds all
the logic; the command line program `linlog` (package `linlog-cli`) in
`cli/` is a thin front end; a web front end will compile the library to
WebAssembly, so the library uses neither threads nor the clock on its own.

The library keeps a sequent as a compact arena of subformulas in negation
normal form, one-sided (`Γ ⊢ Δ` becomes `⊢ Γ^⊥, Δ`); an intuitionistic
sequent is the same arena read two-sided, through the polarization of its
subformulas, with no second representation. On top of that it detects the
fragment a sequent lives in and builds the occurrence forest, the
numbering of subformula occurrences that every proof-search engine, proof
checker and proof net works on. A proof is a compact term over those
occurrences, one node per rule instance; an independent checker decides
whether it proves its sequent, and a derivation view unfolds it into the
tree of explicit sequents of the standard sequent calculus, one-sided or
two-sided. A proof net is the same forest with axiom links, checked by its
own criterion and convertible to and from a proof term. Proof search
decides a sequent, or any goal within one, with the engine its fragment
and mode call for, sequent search, net search or the additive recursion,
and returns a checked proof, that there is none, or why it could not tell.
Interactive proving holds a derivation with open goals over the same
forest, with the same inferences as the derivation view, and turns it back
into a term for the checker once it is finished. The exports write
sequents and derivations, finished or not, as LaTeX and Typst source, one
inference at a time.
