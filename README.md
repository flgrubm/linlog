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
provable (ALL, classical, additive engine)
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
$ linlog prove --mix --stats --deterministic "|- A par B, ~A, ~B"
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

The focused engines treat one literal of every atom as positive, which
decides where a proof keeps its focus and never what is provable; `--bias`
names the rule. The default, `auto`, makes the literal positive that is
more often a direct factor of a `⊗` (`factors`: such a `⊗` needs no search
for its split) when the sequent has no exponential, and the rarer literal
(`rarer`) when it has one or under `--affine`. On Horn-like hypotheses under `!`, a Petri net
for one, `factors` chains forward from the facts and is often faster by
orders of magnitude, but its proofs take one copy per step on a single
branch, so it wants `--copies` raised to the number of steps:

```console
$ linlog prove -q --deterministic --stats "!(a * a -o b), !(b * b -o c), !(c * c -o d), a, a, a, a, a, a, a, a |- d"
provable (MELL, classical, focus engine)
stable sequents visited: 14228 (13935 from the memo)
memo entries at most: 190
splits examined: 42105
time: 2.08ms
$ linlog prove -q --deterministic --stats --bias factors "!(a * a -o b), !(b * b -o c), !(c * c -o d), a, a, a, a, a, a, a, a |- d"
unknown (MELL, classical, focus engine): the copy bound of 3 was reached; raise it with --copies
stable sequents visited: 11 (0 from the memo)
memo entries at most: 5
splits examined: 31
time: 20.01µs
$ linlog prove -q --deterministic --stats --bias factors --copies 7 "!(a * a -o b), !(b * b -o c), !(c * c -o d), a, a, a, a, a, a, a, a |- d"
provable (MELL, classical, focus engine)
stable sequents visited: 47 (5 from the memo)
memo entries at most: 9
splits examined: 151
time: 45.69µs
```

The search runs on every core by default: `--jobs N` (`-j`) sets the
threads, and `--deterministic` runs the sequential engines, whose proof
and statistics are a function of the input, where a parallel run may find
a different proof of the same sequent, never a different verdict. The
focus engine splits the choices nearest the root among the threads and
shares its memo; the net engine splits the first links into cubes; the
additive engine is sequential in every case:

```console
$ linlog prove -q -j 4 "!(A -o A * A), !(B * B -o C), A, B |- A * A * A"
unknown (MELL, classical, focus engine): the copy bound of 3 was reached; raise it with --copies
$ linlog prove -q --deterministic "!(A -o A * A), !(B * B -o C), A, B |- A * A * A"
unknown (MELL, classical, focus engine): the copy bound of 3 was reached; raise it with --copies
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
goals as bare sequents (`show latex`, `show typst` and `show svg` as proof trees), `save` and `load` keep a session as JSON, and
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
to the tree and set in the Euler math font: `pdflatex` needs the
ebproof, cmll, eulervm, amsfonts and standalone packages, `typst compile`
fetches curryst 0.6.0 on first use and needs the
[Euler Math](https://ctan.org/pkg/euler-math) font (`--font-path`). Typst
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

`--format svg` draws the derivation as an SVG image instead, and `--format
net-svg` the proof net of the proof, for the sequents `--format net`
takes: the conclusions at the bottom, every `⊗` and `⅋` a circle (the
premise edges of a `⅋` dashed and blue, since a switching keeps one of
them), and every axiom link an arc over the literals it joins. The
verdict becomes an XML comment. `seq print --format svg` and `show svg` in
`interact` draw a sequent and the derivation so far. The drawings ask for
the Euler Math font without embedding it; every text is stretched to the
width Euler Math gives it, so a viewer without the font keeps the layout.
The text stays selectable, and `--standalone` is refused, since an SVG is
always a document.

```console
$ linlog prove -i --format svg --output proof.svg "1, A & B, B -o C |- C"
$ linlog prove --format net-svg --output net.svg "A * B |- B * A"
$ linlog seq print --format svg "A |- A"
<svg xmlns="http://www.w3.org/2000/svg" width="70.272" height="27.2" viewBox="0 0 4392 1700" font-family="'Euler Math', 'Neo Euler', serif" font-size="1000" fill="black">
<title>⊢ A⊥, A</title>
<g>
<text x="300" y="1190" textLength="1801" lengthAdjust="spacing">⊢ 𝐴</text>
<text x="2101" y="790" textLength="611" lengthAdjust="spacing" font-size="700">⊥</text>
<text x="2712" y="1190" textLength="1380" lengthAdjust="spacing">, 𝐴</text>
</g>
</svg>
```

The first two draw these:

![The derivation of 1, A & B, B ⊸ C ⊢ C](core/tests/snapshots/ill.svg)
![The proof net of ⊢ A⊥ ⅋ B⊥, B ⊗ A](core/tests/snapshots/net.svg)

`--format rocq` writes the derivation as a proof script that the Rocq
kernel [NanoYalla](https://github.com/ComputerAidedLL/click-and-collect/tree/master/nanoyalla)
checks, the kernel of Click & coLLecT: a lemma stating the one-sided
sequent over the atoms as `formula` variables, proved by one derived rule
of the kernel per inference, with an exchange before a `⊗` where the
kernel's list sequents need one, and closed by `Qed`, so Rocq accepts the
file only if the kernel accepts the proof. The verdict becomes a comment,
and `--standalone` adds the import line. A proof with Mix or with the
weakening of affine mode has no certificate, since the kernel has no such
rule; an intuitionistic proof is certified as the classical proof it is.

```console
$ linlog prove --format rocq "A * B |- B * A"
(* provable (MLL, classical, net engine) *)
Lemma certificate (A B : formula) : ll [parr (dual A) (dual B); tens B A].
Proof.
apply (parr_r_ext []); cbn_sequent.
apply (ex_perm_r [2; 0; 1] [dual B; tens B A; dual A]).
apply (tens_r_ext [dual B]); cbn_sequent.
{
  ax_expansion.
}
{
  ax_expansion.
}
Qed.
```

To check a certificate, install NanoYalla 1.1.3 (the `nanoyalla`
directory of the Click & coLLecT repository) with Rocq 9 and its standard
library: `./configure && make && make install` there, or
`rocq compile -R . NanoYalla nanoll.v` and the same for `macroll.v`, then
compile the certificate with the kernel on the load path:

```console
$ linlog prove --format rocq --standalone --output proof.v "!A, !B |- !(A * A)"
$ rocq compile -R path/to/nanoyalla NanoYalla proof.v
```

Rocq prints nothing for a certificate it accepts. The kernel needs no
Yalla installation, and the certificate uses no cut and no axiom.

The syntax: `*`/`⊗` tensor, `|`/`par`/`⅋` par, `&` with, `+`/`⊕` plus,
`-o`/`⊸` linear implication, `~A` or `A^` negation, `!` and `?`, and the
units `1`, `bot`/`⊥`, `top`/`⊤`, `0`; `|-` or `⊢` separates the sides.

### Benchmarks

`linlog-bench` (`cargo run --release -p linlog-bench -- …` in a checkout,
or `nix build .#linlog-bench`) times the engines on three kinds of
problems: generated families with known verdicts (`linlog-bench families`
lists them: 3-Partition as a Horn program and as an MLL sequent,
Matsuoka's Partition, random QBF, wide sequents, contexts under Mix,
Petri-net counters and more, each at any size), the problems of the
[LLTP library](https://github.com/meta-logic/lltp) (`nix build .#lltp -o
bench/lltp` fetches it at a pinned commit; its problems under `ILL/` run
intuitionistically), and problem files of lines `name; mode; expected;
copies; sequent` such as `bench/problems/slow-tests.txt`. `run` runs every
problem in every mode, engine and thread count asked for, each run in a
child process of its own with a time limit, and writes one CSV row per
run with the verdict, the time and the engine's counters; `summary`
prints Markdown tables of CSV files: the problems solved within the time
limit per family and configuration, and a time per problem and
configuration.

```console
$ linlog-bench run --family partition-no=3,4 --engines focus,net --jobs 1,4 --timeout 10 --output runs.csv
[1/8] partition-no/3 classical focus j1: unprovable  0.089 ms, about 0 min left
[2/8] partition-no/3 classical focus j4: unprovable  0.146 ms, about 0 min left
[3/8] partition-no/3 classical net j1: unprovable  3355.671 ms, about 0 min left
[4/8] partition-no/3 classical net j4: unprovable  1020.791 ms, about 0 min left
[5/8] partition-no/4 classical focus j1: unprovable  0.146 ms, about 0 min left
[6/8] partition-no/4 classical focus j4: unprovable  0.303 ms, about 0 min left
[7/8] partition-no/4 classical net j1: unknown timeout 10000.591 ms, about 0 min left
[8/8] partition-no/4 classical net j4: unknown timeout 10000.139 ms, about 0 min left

$ linlog-bench summary runs.csv
…
## partition-no

| problem | runs: classical focus j1 | runs: classical focus j4 | runs: classical net j1 | runs: classical net j4 |
|---|--:|--:|--:|--:|
| partition-no/3 | 89 µs ✗ | 146 µs ✗ | 3.36 s ✗ | 1.02 s ✗ |
| partition-no/4 | 146 µs ✗ | 303 µs ✗ | > 10 s | > 10 s |
```

`run --bias rarer|factors` runs the focused engines under that bias, and
the rows say which. `bench/targets.sh LABEL` runs the target set of the
focused engine's performance work, the instances the first baseline
showed it losing on (the hard families at the sizes that took minutes or
did not finish, and a fixed sample of 113 LLTP problems), on two pinned
cores in a memory-capped user unit, into `bench/targets/LABEL.csv`;
`bench/TARGETS.md` sets the engine before that work beside the engine
after it. On one thread the engine's counters are a function of the
input, so two such files tell whether a change altered the search at all.

`bench/baseline.sh --arm --fresh` takes the whole baseline, every
family, engine and thread count and the whole LLTP library, unattended
in the night: a user timer starts it as a systemd user unit at 20:00 (or
at once if that has passed), where it waits for an otherwise idle
machine, runs about ten and a half hours, and is stopped at 07:00
whatever its state (`--slot=HH:MM-HH:MM` for other times; the script run
again without `--fresh` finishes a stopped baseline on another night).
Every baseline keeps a directory of its own named by the day it started,
`bench/results/DAY/`: the CSV files of its runs, `starts.txt` with the
commit measured, and its tables in `RESULTS.md`, which `bench/RESULTS.md`
copies for the latest baseline; `journalctl --user -fu linlog-baseline`
follows it. A last stage runs again, with more time before the kill and
more memory, the runs that `bench/reruns.txt` lists: those of an earlier
baseline that were killed or crashed and that measurement showed to
finish with more room. The first baseline, of the night of 2026-09-30, took 9 h 21 min
on a 16-core Intel Core Ultra X9 388H, and a supplement on the next
night added that last stage to it in 1 h 28 min; `bench/RESULTS.md` has
its tables.

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
  engine over dyadic sequents of occurrence bitsets with a memo, counts
  that prune sequents and direct the search for the split of a `⊗`
  (contexts of any width), one representative for formulas that occur
  several times, an atom bias chosen from the sequent or by `--bias`, a
  per-branch bound on the copies of `?` formulas that deepens
  iteratively, and a loop check, one-sided or two-sided; for
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
  standalone documents set in Euler, and drawings of sequents,
  derivations and proof nets as SVG, laid out with the character widths
  of the Euler Math font, a switching cycle of an incorrect net
  highlighted.
- Proof certificates: a finished proof as a Rocq script for the NanoYalla
  kernel, a lemma proved rule by rule and closed by `Qed`, for every
  classical fragment and for intuitionistic proofs as the classical
  proofs they are, with a flake check that runs Rocq on them.
- Parallel search, behind the library's `parallel` feature and on by
  default in the command: the focus engine runs the choices nearest the
  root on a thread pool, cube-and-conquer style, with the `&` premises in
  parallel and one memo shared by every thread; the net engine splits the
  first links into cubes for the pool; a stop condition reaches every
  thread; the sequential engines stay one flag away.
- The `linlog` command: `prove`, `check`, `interact` and `seq`, with time
  limits, Ctrl-C, statistics, JSON output, proof nets, LaTeX, Typst,
  SVG and Rocq output, and `--jobs` and `--deterministic` for the search.
- Benchmarks: a reader for the problems of the LLTP library, generated
  families with known verdicts (the hard families of the literature and
  the cases where one engine is known to be slow), and `linlog-bench`,
  which runs them with a time limit per run, writes CSV and summarises it,
  with a script that takes a whole baseline on an idle machine and one
  that runs the focused engine's target set by day.

Planned, in roughly this order:

- Performance work driven by the benchmark numbers.
- Later: proof nets with exponential boxes, essential nets for
  intuitionistic MLL, the inverse method, the Lambek calculus, and a web
  front end.

The design follows [Click and Collect](https://www.click-and-collect.linear-logic.org)
where it is good and departs from it where it is not. Feature requests and
contributions are welcome. All code is licensed under the EUPL.

## Architecture

Three crates, with a fourth to come: the library `linlog` in `core/` holds
all the logic; the command line program `linlog` (package `linlog-cli`) in
`cli/` is a thin front end; the benchmark harness `linlog-bench` in
`bench/` runs the library's engines on problem sets; a web front end will
compile the library to
WebAssembly, so the library uses no clock, and threads only behind its
`parallel` feature, which the web front end leaves off.

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
inference at a time, draw them and proof nets as SVG from layouts of
their own: a tree by subtree widths, a net by its formula trees under the
axiom links, and write a finished derivation as a Rocq proof script for
the NanoYalla kernel, tracking the order of each goal's formulas so that
one exchange per `⊗` suffices.
