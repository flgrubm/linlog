# linlog

A linear logic suite for all your needs: a command line program and a Rust
library that parse, print and decide sequents of classical linear logic and
its fragments, keep the proofs in a checkable form and show them as
derivation trees or proof nets.

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
provable (MLL, classical, focus engine)
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
unprovable (MLL, classical, focus engine): the search was exhaustive
```

`--mix`, `--affine` and `--intuitionistic` choose the logic, `--fragment` and
`--engine` override what detection picks, `--timeout 10s` bounds the search,
`--quiet` prints the verdict line only and `--stats` what the search cost:

```console
$ linlog prove --mix --stats "|- A par B, ~A, ~B"
provable (MLL, classical with Mix, focus engine)
─────── ax   ─────── ax
⊢ A, ~A      ⊢ B, ~B
──────────────────── mix
   ⊢ A, B, ~A, ~B
   ─────────────── ⅋
   ⊢ A ⅋ B, ~A, ~B
stable sequents visited: 3 (0 from the memo)
memo entries at most: 3
splits examined: 4
time: 47.10µs
```

`--format net` shows the proof as a proof net instead of a derivation: the
sequent, the axiom links as pairs of literals with their positions in the
sequent's subformula numbering, and the verdict of the correctness
criterion. Proof nets exist for MLL without units, with or without Mix:

```console
$ linlog prove --format net "|- A * B, C * (~A par ~B), ~C"
provable (MLL, classical, focus engine)
⊢ A ⊗ B, C ⊗ (~A ⅋ ~B), ~C
A[1] — ~A[6]
B[2] — ~B[7]
C[4] — ~C[8]
proof net
$ linlog prove --format net "A & B |- A"
error: proof nets exist for MLL without units only, not for ALL
```

The exit status tells scripts the verdict: 0 provable, 1 unprovable, 3
unknown (the time limit or Ctrl-C stopped the search), 2 an error, such as a
sequent no engine handles yet:

```console
$ linlog prove "!A |- A"
error: no engine for MELL in classical mode yet
```

`--format json` writes the outcome as one JSON object, which is also a proof
file that `linlog check` verifies independently of the search (pass the same
logic flags):

```console
$ linlog prove --format json "A |- A"
{"verdict":"proved","fragment":"MLL","mode":{"intuitionistic":false,"affine":false,"mix":false},"engine":"focus","statistics":{"nodes":1,"memo_hits":0,"memo_entries":1,"splits":0},"sequent":{"terms":[{"D":0},{"V":0}],"ids":[0,1],"var_dict":["A"]},"proof":[{"ax":[0,1]}]}
$ linlog prove --format json "A |- A" | linlog check --quiet
valid proof of ⊢ ~A, A (classical)
```

`linlog seq` prints a sequent one-sided in negation normal form, converts it
to JSON (which `--json-input` reads back), or names its fragment:

```console
$ linlog seq print "A * B -o C |- ~C -o ~(A * B)"
⊢ (A ⊗ B) ⊗ ~C, C ⅋ (~A ⅋ ~B)
$ linlog seq json "A |- A"
{"terms":[{"D":0},{"V":0}],"ids":[0,1],"var_dict":["A"]}
$ linlog seq fragment "A & B |- 1"
MALL
```

The syntax: `*`/`⊗` tensor, `|`/`par`/`⅋` par, `&` with, `+`/`⊕` plus,
`-o`/`⊸` linear implication, `~A` or `A^` negation, `!` and `?`, and the
units `1`, `bot`/`⊥`, `top`/`⊤`, `0`; `|-` or `⊢` separates the sides.

## What exists and what is planned

Built:

- Parsing and printing of sequents in the syntax above, with the ASCII and
  Unicode spellings of every connective, and a compact JSON form.
- Detection of the fragment a sequent lives in (MLL, MLL with units, ALL,
  MALL, MELL, LL) and the modes classical, affine, intuitionistic and Mix
  as user choices.
- Proofs as compact terms over subformula occurrences, an independent
  checker that decides whether a term proves its sequent, and a derivation
  view that unfolds a term into the tree of the standard sequent calculus,
  printed as text.
- Automatic proof search for MLL, MLL with units and MALL, with or without
  Mix: a focused sequent engine over occurrence bitsets with a memo and
  count-based pruning, returning a checked proof, "unprovable" after an
  exhaustive search, or "unknown" with the reason.
- Proof nets for MLL, with or without Mix, as a representation of their
  own: proof structures over the subformula occurrences, an independent
  correctness criterion (Danos–Regnier, decided by Yeo's deletion test on
  the coloured structure graph, with a switching cycle or the
  disconnection named when it fails), sequentialization into a checked
  proof and desequentialization of a proof into its net, a text form and
  a JSON form.
- The `linlog` command: `prove`, `check` and `seq`, with time limits,
  Ctrl-C, statistics, JSON output and proof nets.

Planned, in roughly this order:

- Proof-net search as the engine for unit-free MLL.
- The exponentials (MELL, full LL) with a bounded copy rule, and affine
  mode as a decision procedure.
- Intuitionistic linear logic, with two-sided printing and derivations.
- Export of sequents, derivations and proof nets to LaTeX, Typst and SVG,
  and proof certificates for Rocq.
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
normal form, one-sided (`Γ ⊢ Δ` becomes `⊢ Γ^⊥, Δ`). On top of that it
detects the fragment a sequent lives in and builds the occurrence forest,
the numbering of subformula occurrences that every proof-search engine,
proof checker and proof net works on. A proof is a compact term over those
occurrences, one node per rule instance; an independent checker decides
whether it proves its sequent, and a derivation view unfolds it into the
tree of explicit sequents of the standard sequent calculus. A proof net is
the same forest with axiom links, checked by its own criterion and
convertible to and from a proof term. Proof search decides a sequent with
the engine its fragment calls for and returns a checked proof, that there
is none, or why it could not tell.
