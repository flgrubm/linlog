# linlog

A linear logic suite for all your needs.

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

## Roadmap

Currently planned are a command line program as well as an interactive website (hosted somewhere as a client side website) that provide some functionality of interacting with linear logic sequents and proofs. This tool is mainly meant for educational purposes, but since a major goal is the use of efficient code (within the limits of complexity classes), the tool should be usable for various other use cases.

Please feel free to make feature requests and contribute in any way. All of the code in this repository is licensed under the EUPL.

Planned features:

- Parse and print sequents
- Step-wise interactive sequent proving
- Automatic proof search (different methods for different sub-variants, e.g. MLL)
- Creation and verification of proof nets
- Convert classical/intuitionistic sequents to linear logic
    - solve them and compare proof trees
- Output sequents, proof trees and nets in various formats:
    - PDF/SVG
    - LaTeX/Typst
    - plain Unicode
    - Rocq/Lean/Agda proof
    - interactive web-view
- Further into the future:
    - Intuitionistic linear logic
    - Affine linear logic
- inspired by [Click and Collect](https://www.click-and-collect.linear-logic.org), but more features planned

## Architecture

There will be three units: the CLI program (`linlog`, package `linlog-cli` in `cli/`), the web version (`linlog-web`, not started) as well as a library (`linlog`, in `core/`) for the common logic shared between the CLI and web application. The code is written using Rust, due to its high performance and great compatibility with WebAssembly (for the website).

The library keeps a sequent as a compact arena of subformulas in negation normal form, one-sided (`Γ ⊢ Δ` becomes `⊢ Γ^⊥, Δ`). On top of that it detects the fragment a sequent lives in (MLL, MALL, MELL, LL, …) and builds the occurrence forest, the numbering of subformula occurrences that every proof-search engine, proof checker and proof net works on. A proof is a compact term over those occurrences, one node per rule instance; an independent checker decides whether it proves its sequent, and a derivation view unfolds it into the tree of explicit sequents of the standard sequent calculus, which prints as a text tree and which the exporters will read. Proof search decides a sequent with the engine its fragment calls for and returns a checked proof, that there is none, or why it could not tell; so far that is one focused sequent engine over occurrence bitsets with a memo, for the multiplicative and additive fragments with or without Mix. Proof nets, the exponentials and the export formats follow the plan in `plan/README.md`.
