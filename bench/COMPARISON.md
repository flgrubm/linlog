# The two baselines compared

The baseline of 2026-09-30 (`bench/results/2026-09-30/`, the engines before
the focused engine's performance pass) against that of 2026-10-02
(`bench/results/2026-10-02/`, after it), both taken by `bench/baseline.sh`
on the same machine under the same settings. The tables after "At a
glance" are printed by

```sh
linlog-bench summary --before bench/results/2026-09-30 bench/results/2026-10-02/*.csv
linlog-bench summary --against bench/results/2026-10-02/lltp-intuitionistic.csv \
  bench/results/2026-10-02/lltp-rarer.csv bench/results/2026-10-02/lltp-forward.csv
```

the first comparing every file with the file of the same name in the first
baseline, the second the two passes under one atom bias (new in the
second baseline) with the default pass, problem by problem. A verdict
found after the time limit counts as late and is kept apart.

## At a glance

**No decided verdict differs** between the two baselines on the same
problem in the same configuration, and **no problem decided in the first
baseline is undecided in the second**, in any file.

Families, one thread, 300 s (stage 1 in four streams; the long runs with
1 200 s):

| family | largest decided, first baseline | largest decided, second | a size both decide |
|---|---|---|---|
| 3-partition-no (bins of b) | b = 4: 49.8 s (5 refuted in 302 s at 1 200 s) | every size in under 1 ms: 971 stable sequents whatever b | b = 4: 49.8 s → 299 µs |
| partition-yes (n items) | n = 6: 12.7 s (7 > 1 200 s) | n = 24: 118 s (28 > 300 s and > 1 200 s) | n = 6: 12.7 s → 130 µs |
| partition-no | n = 4: 114 ms (5 refuted in 811 s at 1 200 s) | n = 12: 3.87 s (14 and 15 > 300 s, 15 > 1 200 s) | n = 4: 114 ms → 141 µs |
| qbf (n variables, four instances) | n = 20: three of four, 87–124 s | n = 48: three of four, 230–282 s (#3 > 300 s, #0 out of memory after 290 s) | n = 20 #0: 87.2 s → 7.3 ms |
| wide-m3 (k literals) | k = 30: 10.6 s (36 in 697 s) | k = 1 024: 55 ms (2 048 at the recursion limit) | k = 30: 10.6 s → 112 µs |
| wide-m4 | k = 32: 43.4 s (36 in 685 s) | k = 1 024: 52 ms (2 048 at the recursion limit) | k = 32: 43.4 s → 119 µs |
| mix (k pairs) | k = 10: 214 s (11 > 1 200 s) | k = 10: 146 s (11 > 1 200 s) | k = 9: 19.8 s → 14.1 s |
| counter (n tokens) | n = 8: 124 ms (16 > 1 200 s) | n = 64: 80.7 s (two-sided 268 ms) | n = 8: 124 ms → 87 µs |
| counter-over | copy bound at every size, 8 after 493 ms (16 > 300 s) | refuted up to 16 (299 µs); copy bound at 32 (2.2 s) and 64 (85 s) | – |
| chain (k clauses) | k = 256: 2.82 s | k = 256: 321 ms | as left |
| additive (depth d) | d = 16: 526 ms | d = 16: 210 ms | as left |
| growing | copy bound or recursion limit, as constructed | the same | 1 024: 568 ms → 257 ms |

Focus against net on unit-free MLL (60 s): the focused engine is now as
fast as the net engine or faster on every MLL family, by up to six
orders of magnitude on the Horn encodings (Partition with six items
12.2 s → 130 µs, the net engine still > 60 s), and loses only where it
meets the recursion limit, the wide sequents at 2 048 literals (net
0.64 s). On the wide sequents with 8 to 1 024 literals the two are
within a factor of three (wide-m3 at 256: focus 3.7 ms, net 9.7 ms).

LLTP, one thread, 5 s (decided; late: those of them proved after the
limit):

| file | first baseline | second baseline | of which Petri nets |
|---|--:|--:|---|
| `lltp-intuitionistic` (4 495 problems) | 737 | 2 049 (2 late) | 210 → 1 520 (2 late) |
| `lltp-classical` (4 512) | 742 | 2 008 (2 late) | 204 → 1 468 (2 late) |
| `lltp-rarer` (new: the backward search alone) | – | 969 | 442 |
| `lltp-forward` (new: the forward search alone, 30 copies) | – | 2 393 (2 late) | 1 576 (2 late) |
| `lltp-copies-10` (1 003 that ended at the copy bound) | 411 | 458 | 59 → 89 |
| `lltp-recursion` (995 that ended at the recursion limit) | 56 | 280 | 56 → 280 |
| `lltp-all-cores` (1 102, every core) | 32 (3 late) | 625 (2 late) | |
| `lltp-portfolio` (the same, with the portfolio) | 34 (4 late) | 629 (2 late) | |

Where the intuitionistic pass ended the undecided: first baseline 984
at the time limit, 898 at the copy bound, 983 at the recursion limit, 845
too wide, 47 killed, 1 not loading; second baseline 1 636 at the time
limit, 727 at the copy bound, 54 at the recursion limit, 15 killed, 14 out
of memory. Those 14 are nets of tens of thousands of transitions whose
search finds a proof in 50 ms to 2 s within 250 MB (the command with
`--quiet`); what runs out of the 12 GiB is the harness's check of that
proof, so their rows say `crash` where the search's answer was `proved`.
The 14 `crash` rows of the classical and of the forward pass, the 3 of
the backward pass and 13 of the 17 of `lltp-recursion` are the same nets
(the other four, Philosophers-10000 nets under the raised recursion
limit, run out in the search, as `qbf/48#0` does, whose memo grows for
290 s). Three refutations under the larger copy
bounds contradict their headers beyond the 25 files the first baseline
found (`MISMATCH` below): KLE069 in `KLE-01` and KLE078 and KLE086 in
`KLE-cbn`, each a translated formula with a classical countermodel.

The default against its two components (intuitionistic library, one
thread, 5 s):

| | default | `--bias rarer` | `--bias factors --copies 30` |
|---|--:|--:|--:|
| Petri nets decided | 1 520 | 442 | 1 576 |
| of which the default does not decide | – | 2 (at the limit) | 67 (time limit) |
| decided by the default and not by it | – | 1 080 | 11 |
| default's time over it, median of both decided | – | 1.29× (440) | 1.64× (1 507) |
| other collections decided | 529 | 527 | 817 |
| of which the default does not decide | – | 0 | 288 (all at the default's copy bound of 3) |

Every core against one thread, on the 1 102 problems of
`lltp-all-cores`: 623 decided within 5 s on sixteen threads against 586
on one, 37 gained and none lost, 2.88 times slower in the median on
those both decide. The portfolio against every core: 627 against 623, 10
gained and 6 lost, the same time in the median (1.01×).

## The second baseline against the first

### Verdicts that differ

None.

### Decided by before, not by after

None.

### By group

Problems in both; decided (late: after the time limit); decided by one and not the other; where the undecided ended (`time` the time limit or `killed` past it, `bound` the copy bound, `depth` the recursion limit, `wide` a context too wide to split, `crash` out of memory); the median ratio of after's time to before's on the problems both decide within the limit (how many); and the problems only after has.

| file | family | configuration | problems | before decided | after decided | only before | only after | before ended | after ended | after/before | new |
|---|---|---|--:|--:|--:|--:|--:|---|---|--:|--:|
| engines | 3-partition-mll-no | classical focus j1 | 4 | 4 | 4 | 0 | 0 | – | – | 1.29× (4) | 0 |
| engines | 3-partition-mll-no | classical net j1 | 4 | 2 | 2 | 0 | 0 | 2 time | 2 time | 1.04× (2) | 0 |
| engines | 3-partition-mll-yes | classical focus j1 | 4 | 4 | 4 | 0 | 0 | – | – | 1.35× (4) | 0 |
| engines | 3-partition-mll-yes | classical net j1 | 4 | 3 | 3 | 0 | 0 | 1 time | 1 time | 1.02× (3) | 0 |
| engines | additive | classical focus j1 | 4 | 3 | 3 | 0 | 0 | 1 crash | 1 crash | 0.90× (3) | 0 |
| engines | cancellation | classical focus j1 | 1 | 1 | 1 | 0 | 0 | – | – | 6.3e-6× (1) | 0 |
| engines | partition-no | classical focus j1 | 2 | 2 | 2 | 0 | 0 | – | – | 1.4e-2× (2) | 2 |
| engines | partition-no | classical net j1 | 2 | 1 | 1 | 0 | 0 | 1 time | 1 time | 1.04× (1) | 2 |
| engines | partition-table | classical focus j1 | 13 | 12 | 13 | 0 | 1 | 1 time | – | 2.5e-2× (12) | 0 |
| engines | partition-table | classical net j1 | 13 | 9 | 9 | 0 | 0 | 4 time | 4 time | 1.02× (9) | 0 |
| engines | partition-yes | classical focus j1 | 3 | 3 | 3 | 0 | 0 | – | – | 4.6e-4× (3) | 2 |
| engines | partition-yes | classical net j1 | 3 | 1 | 1 | 0 | 0 | 2 time | 2 time | 1.12× (1) | 2 |
| engines | wide-m1 | classical focus j1 | 6 | 4 | 5 | 0 | 1 | 2 wide | 1 depth | 8.9e-2× (4) | 0 |
| engines | wide-m1 | classical net j1 | 6 | 6 | 6 | 0 | 0 | – | – | 0.99× (6) | 0 |
| engines | wide-m2 | classical focus j1 | 6 | 4 | 5 | 0 | 1 | 2 wide | 1 depth | 9.2e-2× (4) | 0 |
| engines | wide-m2 | classical net j1 | 6 | 6 | 6 | 0 | 0 | – | – | 0.98× (6) | 0 |
| engines | wide-m3 | classical focus j1 | 3 | 3 | 3 | 0 | 0 | – | – | 5.6e-4× (3) | 2 |
| engines | wide-m3 | classical net j1 | 3 | 3 | 3 | 0 | 0 | – | – | 0.98× (3) | 2 |
| engines | wide-m4 | classical focus j1 | 3 | 3 | 3 | 0 | 0 | – | – | 5.4e-4× (3) | 2 |
| engines | wide-m4 | classical net j1 | 3 | 3 | 3 | 0 | 0 | – | – | 0.96× (3) | 2 |
| families | 3-partition-mll-no | classical auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 1.25× (4) | 0 |
| families | 3-partition-mll-yes | classical auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 1.33× (4) | 0 |
| families | 3-partition-no | classical auto j1 | 2 | 1 | 2 | 0 | 1 | 1 time | – | 6.0e-6× (1) | 0 |
| families | 3-partition-yes | classical auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 7.7e-2× (4) | 0 |
| families | additive | classical auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 1.01× (4) | 0 |
| families | chain | classical auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 0.23× (4) | 0 |
| families | counter | classical auto j1 | 4 | 3 | 4 | 0 | 1 | 1 time | – | 0.77× (3) | 2 |
| families | counter-over | classical auto j1 | 4 | 0 | 4 | 0 | 4 | 1 time, 3 bound | – | – | 2 |
| families | growing | classical auto j1 | 4 | 0 | 0 | 0 | 0 | 3 bound, 1 depth | 3 bound, 1 depth | – | 0 |
| families | mix | mix auto j1 | 6 | 5 | 5 | 0 | 0 | 1 time | 1 time | 0.81× (5) | 0 |
| families | partition-no | classical auto j1 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 1.3e-2× (2) | 4 |
| families | partition-yes | classical auto j1 | 4 | 3 | 4 | 0 | 1 | 1 time | – | 4.1e-4× (3) | 5 |
| families | qbf | classical auto j1 | 20 | 15 | 20 | 0 | 5 | 5 time | – | 1.5e-2× (15) | 16 |
| families | wide-m1 | classical auto j1 | 6 | 6 | 6 | 0 | 0 | – | – | 0.95× (6) | 0 |
| families | wide-m2 | classical auto j1 | 6 | 6 | 6 | 0 | 0 | – | – | 0.97× (6) | 0 |
| families | wide-m3 | classical auto j1 | 4 | 3 | 4 | 0 | 1 | 1 time | – | 5.3e-4× (3) | 3 |
| families | wide-m4 | classical auto j1 | 5 | 4 | 5 | 0 | 1 | 1 time | – | 5.0e-4× (4) | 3 |
| intuitionistic | 3-partition-no | intuitionistic auto j1 | 1 | 1 | 1 | 0 | 0 | – | – | 1.2e-5× (1) | 0 |
| intuitionistic | 3-partition-yes | intuitionistic auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 0.14× (4) | 0 |
| intuitionistic | chain | intuitionistic auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 0.24× (4) | 0 |
| intuitionistic | counter | intuitionistic auto j1 | 4 | 3 | 4 | 0 | 1 | 1 time | – | 1.27× (3) | 2 |
| intuitionistic | counter-over | intuitionistic auto j1 | 4 | 0 | 4 | 0 | 4 | 1 time, 3 bound | – | – | 2 |
| lltp-all-cores | ILL/ILLTP-SYJ-01 | intuitionistic auto j16 | 31 | 0 | 0 | 0 | 0 | 1 time, 13 bound, 12 depth, 2 killed, 2 crash, 1 error | 13 bound, 12 depth, 2 killed, 4 crash | – | 0 |
| lltp-all-cores | ILL/ILLTP-SYJ-cbn | intuitionistic auto j16 | 31 | 0 | 0 | 0 | 0 | 15 bound, 12 depth, 1 killed, 3 crash | 16 bound, 12 depth, 2 killed, 1 crash | – | 0 |
| lltp-all-cores | ILL/ILLTP-SYJ-cbv | intuitionistic auto j16 | 31 | 1 | 2 | 0 | 1 | 11 time, 19 killed | 10 time, 19 killed | 0.44× (1) | 0 |
| lltp-all-cores | ILL/petri-nets/MCC | intuitionistic auto j16 | 1009 | 31 (3 late) | 623 (2 late) | 0 | 592 | 769 time, 20 bound, 2 wide, 187 killed | 386 time | 1.1e-2× (28) | 0 |
| lltp-all-cores-generous | ILL/ILLTP-SYJ-01 | intuitionistic auto j16 | 5 | 0 | 0 | 0 | 0 | 5 time | 4 time, 1 bound | – | 0 |
| lltp-all-cores-generous | ILL/ILLTP-SYJ-cbn | intuitionistic auto j16 | 4 | 0 | 0 | 0 | 0 | 2 time, 2 bound | 2 time, 2 bound | – | 0 |
| lltp-all-cores-generous | ILL/ILLTP-SYJ-cbv | intuitionistic auto j16 | 3 | 0 | 0 | 0 | 0 | 3 time | 3 time | – | 0 |
| lltp-classical | CLL/Non-theorems | classical auto j1 | 3 | 3 | 3 | 0 | 0 | – | – | 1.21× (3) | 0 |
| lltp-classical | CLL/misc | classical auto j1 | 14 | 8 | 8 | 0 | 0 | 1 time, 5 bound | 6 bound | 1.23× (8) | 0 |
| lltp-classical | ILL/ILLTP-LCL-01 | classical auto j1 | 2 | 0 | 0 | 0 | 0 | 2 bound | 2 bound | – | 0 |
| lltp-classical | ILL/ILLTP-LCL-cbn | classical auto j1 | 2 | 1 | 2 | 0 | 1 | 1 bound | – | 1.26× (1) | 0 |
| lltp-classical | ILL/ILLTP-LCL-cbv | classical auto j1 | 2 | 0 | 0 | 0 | 0 | 2 bound | 2 bound | – | 0 |
| lltp-classical | ILL/ILLTP-SYJ-01 | classical auto j1 | 252 | 11 | 11 | 0 | 0 | 6 time, 209 bound, 22 depth, 3 killed, 1 error | 7 time, 209 bound, 22 depth, 3 killed | 1.27× (11) | 0 |
| lltp-classical | ILL/ILLTP-SYJ-cbn | classical auto j1 | 252 | 34 | 35 | 0 | 1 | 8 time, 186 bound, 22 depth, 2 killed | 8 time, 185 bound, 22 depth, 2 killed | 3.00× (34) | 0 |
| lltp-classical | ILL/ILLTP-SYJ-cbv | classical auto j1 | 252 | 39 | 39 | 0 | 0 | 27 time, 173 bound, 10 depth, 3 killed | 27 time, 173 bound, 10 depth, 3 killed | 1.24× (39) | 0 |
| lltp-classical | ILL/ILLTP-SYN-01 | classical auto j1 | 19 | 11 | 11 | 0 | 0 | 8 bound | 8 bound | 1.00× (11) | 0 |
| lltp-classical | ILL/ILLTP-SYN-cbn | classical auto j1 | 19 | 16 | 16 | 0 | 0 | 3 bound | 3 bound | 1.00× (16) | 0 |
| lltp-classical | ILL/ILLTP-SYN-cbv | classical auto j1 | 19 | 16 | 16 | 0 | 0 | 3 bound | 3 bound | 1.00× (16) | 0 |
| lltp-classical | ILL/KLE-01 | classical auto j1 | 88 | 50 | 50 | 0 | 0 | 38 bound | 38 bound | 0.98× (50) | 0 |
| lltp-classical | ILL/KLE-IMP-CONJ | classical auto j1 | 222 | 162 | 162 | 0 | 0 | 60 bound | 60 bound | 1.03× (162) | 0 |
| lltp-classical | ILL/KLE-IMP-CONJ/ALT | classical auto j1 | 27 | 27 | 27 | 0 | 0 | – | – | 0.95× (27) | 0 |
| lltp-classical | ILL/KLE-IMP-CONJ/NON-THEOREMS | classical auto j1 | 22 | 22 | 22 | 0 | 0 | – | – | 1.10× (22) | 0 |
| lltp-classical | ILL/KLE-cbn | classical auto j1 | 88 | 68 | 68 | 0 | 0 | 20 bound | 20 bound | 1.53× (68) | 0 |
| lltp-classical | ILL/KLE-cbv | classical auto j1 | 88 | 67 | 67 | 0 | 0 | 21 bound | 21 bound | 1.31× (67) | 0 |
| lltp-classical | ILL/Non-theorems | classical auto j1 | 1 | 0 | 0 | 0 | 0 | 1 bound | 1 bound | – | 0 |
| lltp-classical | ILL/misc | classical auto j1 | 3 | 3 | 3 | 0 | 0 | – | – | 1.35× (3) | 0 |
| lltp-classical | ILL/petri-nets/MCC | classical auto j1 | 3137 | 204 | 1468 (2 late) | 0 | 1264 | 1049 time, 68 bound, 929 depth, 845 wide, 42 killed | 1646 time, 2 bound, 7 killed, 14 crash | 0.42× (204) | 0 |
| lltp-classical-generous | ILL/ILLTP-SYJ-01 | classical auto j1 | 4 | 0 | 0 | 0 | 0 | 4 time | 4 time | – | 0 |
| lltp-classical-generous | ILL/ILLTP-SYJ-cbn | classical auto j1 | 2 | 0 | 0 | 0 | 0 | 2 time | 2 time | – | 0 |
| lltp-classical-generous | ILL/ILLTP-SYJ-cbv | classical auto j1 | 3 | 0 | 0 | 0 | 0 | 3 time | 3 time | – | 0 |
| lltp-classical-generous | ILL/petri-nets/MCC | classical auto j1 | 25 | 0 | 15 | 0 | 15 | 18 time, 2 killed, 5 crash | 10 time | – | 0 |
| lltp-copies-10 | ILL/ILLTP-LCL-01 | intuitionistic auto j1 | 2 | 1 | 1 | 0 | 0 | 1 bound | 1 bound | 1.09× (1) | 0 |
| lltp-copies-10 | ILL/ILLTP-LCL-cbn | intuitionistic auto j1 | 2 | 2 | 2 | 0 | 0 | – | – | 2.18× (2) | 0 |
| lltp-copies-10 | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 | 2 | 1 | 1 | 0 | 0 | 1 bound | 1 bound | 1.25× (1) | 0 |
| lltp-copies-10 | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 209 | 37 | 37 | 0 | 0 | 107 time, 65 bound | 107 time, 65 bound | 0.88× (37) | 0 |
| lltp-copies-10 | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 209 | 60 | 75 | 0 | 15 | 100 time, 49 bound | 88 time, 46 bound | 1.44× (60) | 0 |
| lltp-copies-10 | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 209 | 83 | 83 | 0 | 0 | 100 time, 26 bound | 98 time, 28 bound | 0.98× (83) | 0 |
| lltp-copies-10 | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | 8 | 5 | 5 | 0 | 0 | 3 bound | 3 bound | 1.07× (5) | 0 |
| lltp-copies-10 | ILL/ILLTP-SYN-cbn | intuitionistic auto j1 | 8 | 5 | 5 | 0 | 0 | 3 bound | 3 bound | 1.38× (5) | 0 |
| lltp-copies-10 | ILL/ILLTP-SYN-cbv | intuitionistic auto j1 | 8 | 5 | 5 | 0 | 0 | 3 bound | 3 bound | 1.14× (5) | 0 |
| lltp-copies-10 | ILL/KLE-01 | intuitionistic auto j1 | 38 | 29 | 29 | 0 | 0 | 9 bound | 9 bound | 1.06× (29) | 0 |
| lltp-copies-10 | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | 60 | 60 | 60 | 0 | 0 | – | – | 1.10× (60) | 0 |
| lltp-copies-10 | ILL/KLE-cbn | intuitionistic auto j1 | 38 | 32 | 34 | 0 | 2 | 6 bound | 4 bound | 1.49× (32) | 0 |
| lltp-copies-10 | ILL/KLE-cbv | intuitionistic auto j1 | 38 | 32 | 32 | 0 | 0 | 6 bound | 6 bound | 1.11× (32) | 0 |
| lltp-copies-10 | ILL/Non-theorems | intuitionistic auto j1 | 1 | 0 | 0 | 0 | 0 | 1 bound | 1 bound | – | 0 |
| lltp-copies-10 | ILL/petri-nets/MCC | intuitionistic auto j1 | 171 | 59 | 89 | 0 | 30 | 108 time, 4 bound | 80 time, 2 bound | 8.6e-3× (59) | 0 |
| lltp-intuitionistic | ILL/ILLTP-LCL-01 | intuitionistic auto j1 | 2 | 0 | 0 | 0 | 0 | 2 bound | 2 bound | – | 0 |
| lltp-intuitionistic | ILL/ILLTP-LCL-cbn | intuitionistic auto j1 | 2 | 1 | 2 | 0 | 1 | 1 bound | – | 1.10× (1) | 0 |
| lltp-intuitionistic | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 | 2 | 0 | 0 | 0 | 0 | 2 bound | 2 bound | – | 0 |
| lltp-intuitionistic | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 252 | 11 | 11 | 0 | 0 | 6 time, 209 bound, 22 depth, 3 killed, 1 error | 7 time, 209 bound, 22 depth, 3 killed | 1.21× (11) | 0 |
| lltp-intuitionistic | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 252 | 34 | 35 | 0 | 1 | 8 time, 186 bound, 22 depth, 2 killed | 8 time, 185 bound, 22 depth, 2 killed | 2.71× (34) | 0 |
| lltp-intuitionistic | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 252 | 39 | 39 | 0 | 0 | 27 time, 173 bound, 10 depth, 3 killed | 27 time, 173 bound, 10 depth, 3 killed | 1.14× (39) | 0 |
| lltp-intuitionistic | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | 19 | 11 | 11 | 0 | 0 | 8 bound | 8 bound | 1.23× (11) | 0 |
| lltp-intuitionistic | ILL/ILLTP-SYN-cbn | intuitionistic auto j1 | 19 | 16 | 16 | 0 | 0 | 3 bound | 3 bound | 1.42× (16) | 0 |
| lltp-intuitionistic | ILL/ILLTP-SYN-cbv | intuitionistic auto j1 | 19 | 16 | 16 | 0 | 0 | 3 bound | 3 bound | 1.25× (16) | 0 |
| lltp-intuitionistic | ILL/KLE-01 | intuitionistic auto j1 | 88 | 50 | 50 | 0 | 0 | 38 bound | 38 bound | 1.18× (50) | 0 |
| lltp-intuitionistic | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | 222 | 162 | 162 | 0 | 0 | 60 bound | 60 bound | 1.19× (162) | 0 |
| lltp-intuitionistic | ILL/KLE-IMP-CONJ/ALT | intuitionistic auto j1 | 27 | 27 | 27 | 0 | 0 | – | – | 1.18× (27) | 0 |
| lltp-intuitionistic | ILL/KLE-IMP-CONJ/NON-THEOREMS | intuitionistic auto j1 | 22 | 22 | 22 | 0 | 0 | – | – | 1.00× (22) | 0 |
| lltp-intuitionistic | ILL/KLE-cbn | intuitionistic auto j1 | 88 | 68 | 68 | 0 | 0 | 20 bound | 20 bound | 1.37× (68) | 0 |
| lltp-intuitionistic | ILL/KLE-cbv | intuitionistic auto j1 | 88 | 67 | 67 | 0 | 0 | 21 bound | 21 bound | 1.18× (67) | 0 |
| lltp-intuitionistic | ILL/Non-theorems | intuitionistic auto j1 | 1 | 0 | 0 | 0 | 0 | 1 bound | 1 bound | – | 0 |
| lltp-intuitionistic | ILL/misc | intuitionistic auto j1 | 3 | 3 | 3 | 0 | 0 | – | – | 1.33× (3) | 0 |
| lltp-intuitionistic | ILL/petri-nets/MCC | intuitionistic auto j1 | 3137 | 210 | 1520 (2 late) | 0 | 1310 | 943 time, 171 bound, 929 depth, 845 wide, 39 killed | 1594 time, 2 bound, 7 killed, 14 crash | 0.52× (210) | 0 |
| lltp-intuitionistic-generous | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 4 | 0 | 0 | 0 | 0 | 4 time | 4 time | – | 0 |
| lltp-intuitionistic-generous | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 2 | 0 | 0 | 0 | 0 | 2 time | 2 time | – | 0 |
| lltp-intuitionistic-generous | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 3 | 0 | 0 | 0 | 0 | 3 time | 3 time | – | 0 |
| lltp-intuitionistic-generous | ILL/petri-nets/MCC | intuitionistic auto j1 | 19 | 0 | 11 | 0 | 11 | 19 time | 8 time | – | 0 |
| lltp-portfolio | ILL/ILLTP-SYJ-01 | intuitionistic auto j16 portfolio | 31 | 0 | 0 | 0 | 0 | 1 time, 13 bound, 12 depth, 2 killed, 2 crash, 1 error | 13 bound, 12 depth, 2 killed, 4 crash | – | 0 |
| lltp-portfolio | ILL/ILLTP-SYJ-cbn | intuitionistic auto j16 portfolio | 31 | 0 | 0 | 0 | 0 | 15 bound, 12 depth, 1 killed, 3 crash | 15 bound, 12 depth, 2 killed, 2 crash | – | 0 |
| lltp-portfolio | ILL/ILLTP-SYJ-cbv | intuitionistic auto j16 portfolio | 31 | 1 | 2 | 0 | 1 | 11 time, 19 killed | 10 time, 19 killed | 0.44× (1) | 0 |
| lltp-portfolio | ILL/petri-nets/MCC | intuitionistic auto j16 portfolio | 1009 | 33 (4 late) | 627 (2 late) | 0 | 594 | 767 time, 20 bound, 2 wide, 187 killed | 382 time | 9.2e-3× (29) | 0 |
| lltp-portfolio-generous | ILL/ILLTP-SYJ-01 | intuitionistic auto j16 portfolio | 5 | 0 | 0 | 0 | 0 | 5 time | 4 time, 1 bound | – | 0 |
| lltp-portfolio-generous | ILL/ILLTP-SYJ-cbn | intuitionistic auto j16 portfolio | 4 | 0 | 0 | 0 | 0 | 2 time, 2 bound | 2 time, 2 bound | – | 0 |
| lltp-portfolio-generous | ILL/ILLTP-SYJ-cbv | intuitionistic auto j16 portfolio | 3 | 0 | 0 | 0 | 0 | 3 time | 3 time | – | 0 |
| lltp-recursion | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 22 | 0 | 0 | 0 | 0 | 22 time | 20 time, 2 depth | – | 0 |
| lltp-recursion | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 22 | 0 | 0 | 0 | 0 | 20 time, 2 depth | 20 time, 2 depth | – | 0 |
| lltp-recursion | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 22 | 0 | 0 | 0 | 0 | 22 time | 22 time | – | 0 |
| lltp-recursion | ILL/petri-nets/MCC | intuitionistic auto j1 | 929 | 56 | 280 | 0 | 224 | 311 time, 152 depth, 401 wide, 9 killed | 632 time, 17 crash | 0.20× (56) | 0 |
| lltp-recursion-generous | ILL/petri-nets/MCC | intuitionistic auto j1 | 9 | 2 (2 late) | 5 | 0 | 3 | 5 killed, 2 crash | 4 time | – | 0 |
| long-1 | 3-partition-no | classical auto j1 | 1 | 1 | 1 | 0 | 0 | – | – | 1.3e-6× (1) | 0 |
| long-1 | counter | classical auto j1 | 1 | 0 | 1 | 0 | 1 | 1 time | – | – | 1 |
| long-1 | partition-no | classical auto j1 | 1 | 1 | 1 | 0 | 0 | – | – | 5.9e-7× (1) | 1 |
| long-1 | partition-yes | classical auto j1 | 1 | 0 | 1 | 0 | 1 | 1 time | – | – | 1 |
| long-2 | mix | mix auto j1 | 1 | 0 | 0 | 0 | 0 | 1 time | 1 time | – | 0 |
| long-2 | qbf | classical auto j1 | 1 | 0 | 1 | 0 | 1 | 1 time | – | – | 1 |
| long-2 | wide-m3 | classical auto j1 | 1 | 1 | 1 | 0 | 0 | – | – | 2.3e-7× (1) | 0 |
| long-2 | wide-m4 | classical auto j1 | 1 | 1 | 1 | 0 | 0 | – | – | 2.1e-7× (1) | 0 |
| parallel | 3-partition-no | classical auto j1 | 2 | 1 | 2 | 0 | 1 | 1 time | – | 8.8e-6× (1) | 0 |
| parallel | 3-partition-no | classical auto j16 | 2 | 2 | 2 | 0 | 0 | – | – | 1.2e-4× (2) | 0 |
| parallel | 3-partition-no | classical auto j2 | 2 | 1 | 2 | 0 | 1 | 1 time | – | 1.9e-5× (1) | 0 |
| parallel | 3-partition-no | classical auto j4 | 2 | 2 | 2 | 0 | 0 | – | – | 2.7e-5× (2) | 0 |
| parallel | 3-partition-no | classical auto j8 | 2 | 2 | 2 | 0 | 0 | – | – | 6.3e-5× (2) | 0 |
| parallel | 3-partition-yes | classical auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 8.6e-2× (4) | 0 |
| parallel | 3-partition-yes | classical auto j16 | 4 | 4 | 4 | 0 | 0 | – | – | 0.36× (4) | 0 |
| parallel | 3-partition-yes | classical auto j2 | 4 | 4 | 4 | 0 | 0 | – | – | 0.15× (4) | 0 |
| parallel | 3-partition-yes | classical auto j4 | 4 | 4 | 4 | 0 | 0 | – | – | 0.20× (4) | 0 |
| parallel | 3-partition-yes | classical auto j8 | 4 | 4 | 4 | 0 | 0 | – | – | 0.28× (4) | 0 |
| parallel | counter | classical auto j1 | 4 | 3 | 4 | 0 | 1 | 1 time | – | 0.60× (3) | 2 |
| parallel | counter | classical auto j16 | 4 | 3 | 4 | 0 | 1 | 1 time | – | 0.87× (3) | 2 |
| parallel | counter | classical auto j2 | 4 | 3 | 4 | 0 | 1 | 1 time | – | 0.44× (3) | 2 |
| parallel | counter | classical auto j4 | 4 | 3 | 4 | 0 | 1 | 1 time | – | 0.62× (3) | 2 |
| parallel | counter | classical auto j8 | 4 | 3 | 4 | 0 | 1 | 1 time | – | 0.85× (3) | 2 |
| parallel | counter-over | classical auto j1 | 4 | 0 | 4 | 0 | 4 | 1 time, 3 bound | – | – | 2 |
| parallel | counter-over | classical auto j16 | 4 | 0 | 4 | 0 | 4 | 1 time, 3 bound | – | – | 2 |
| parallel | counter-over | classical auto j2 | 4 | 0 | 4 | 0 | 4 | 1 time, 3 bound | – | – | 2 |
| parallel | counter-over | classical auto j4 | 4 | 0 | 4 | 0 | 4 | 1 time, 3 bound | – | – | 2 |
| parallel | counter-over | classical auto j8 | 4 | 0 | 4 | 0 | 4 | 1 time, 3 bound | – | – | 2 |
| parallel | mix | mix auto j1 | 4 | 2 | 2 | 0 | 0 | 2 time | 2 time | 0.86× (2) | 0 |
| parallel | mix | mix auto j16 | 4 | 2 | 2 | 0 | 0 | 2 time | 2 time | 1.29× (2) | 0 |
| parallel | mix | mix auto j2 | 4 | 2 | 2 | 0 | 0 | 2 time | 2 time | 1.01× (2) | 0 |
| parallel | mix | mix auto j4 | 4 | 2 | 2 | 0 | 0 | 2 time | 2 time | 1.16× (2) | 0 |
| parallel | mix | mix auto j8 | 4 | 2 | 2 | 0 | 0 | 2 time | 2 time | 1.24× (2) | 0 |
| parallel | partition-no | classical auto j1 | 2 | 1 | 2 | 0 | 1 | 1 time | – | 1.3e-3× (1) | 2 |
| parallel | partition-no | classical auto j16 | 2 | 1 | 2 | 0 | 1 | 1 time | – | 2.7e-2× (1) | 2 |
| parallel | partition-no | classical auto j2 | 2 | 1 | 2 | 0 | 1 | 1 time | – | 3.7e-3× (1) | 2 |
| parallel | partition-no | classical auto j4 | 2 | 1 | 2 | 0 | 1 | 1 time | – | 5.3e-3× (1) | 2 |
| parallel | partition-no | classical auto j8 | 2 | 1 | 2 | 0 | 1 | 1 time | – | 1.3e-2× (1) | 2 |
| parallel | partition-yes | classical auto j1 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 5.1e-4× (2) | 2 |
| parallel | partition-yes | classical auto j16 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 4.0e-2× (2) | 2 |
| parallel | partition-yes | classical auto j2 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 1.1e-3× (2) | 2 |
| parallel | partition-yes | classical auto j4 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 1.3e-3× (2) | 2 |
| parallel | partition-yes | classical auto j8 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 1.3e-2× (2) | 2 |
| parallel | qbf | classical auto j1 | 12 | 7 | 12 | 0 | 5 | 5 time | – | 1.4e-3× (7) | 8 |
| parallel | qbf | classical auto j16 | 12 | 7 | 12 | 0 | 5 | 5 time | – | 2.4e-3× (7) | 8 |
| parallel | qbf | classical auto j2 | 12 | 7 | 12 | 0 | 5 | 5 time | – | 2.2e-3× (7) | 8 |
| parallel | qbf | classical auto j4 | 12 | 7 | 12 | 0 | 5 | 5 time | – | 2.2e-3× (7) | 8 |
| parallel | qbf | classical auto j8 | 12 | 7 | 12 | 0 | 5 | 5 time | – | 2.2e-3× (7) | 8 |
| parallel | wide-m3 | classical auto j1 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 6.1e-4× (2) | 0 |
| parallel | wide-m3 | classical auto j16 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 5.1e-3× (2) | 0 |
| parallel | wide-m3 | classical auto j2 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 1.0e-3× (2) | 0 |
| parallel | wide-m3 | classical auto j4 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 1.7e-3× (2) | 0 |
| parallel | wide-m3 | classical auto j8 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 2.4e-3× (2) | 0 |
| parallel | wide-m4 | classical auto j1 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 5.0e-5× (2) | 0 |
| parallel | wide-m4 | classical auto j16 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 3.5e-4× (2) | 0 |
| parallel | wide-m4 | classical auto j2 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 7.8e-5× (2) | 0 |
| parallel | wide-m4 | classical auto j4 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 1.2e-4× (2) | 0 |
| parallel | wide-m4 | classical auto j8 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 1.7e-4× (2) | 0 |
| parallel-net | 3-partition-mll-no | classical net j1 | 3 | 2 | 2 | 0 | 0 | 1 time | 1 time | 1.07× (2) | 0 |
| parallel-net | 3-partition-mll-no | classical net j16 | 3 | 2 | 2 | 0 | 0 | 1 time | 1 time | 0.98× (2) | 0 |
| parallel-net | 3-partition-mll-no | classical net j2 | 3 | 2 | 2 | 0 | 0 | 1 time | 1 time | 1.08× (2) | 0 |
| parallel-net | 3-partition-mll-no | classical net j4 | 3 | 2 | 2 | 0 | 0 | 1 time | 1 time | 1.06× (2) | 0 |
| parallel-net | 3-partition-mll-no | classical net j8 | 3 | 2 | 2 | 0 | 0 | 1 time | 1 time | 1.02× (2) | 0 |
| parallel-net | partition-table | classical net j1 | 13 | 9 | 9 | 0 | 0 | 4 time | 4 time | 0.98× (9) | 0 |
| parallel-net | partition-table | classical net j16 | 13 | 10 | 10 | 0 | 0 | 3 time | 3 time | 0.98× (10) | 0 |
| parallel-net | partition-table | classical net j2 | 13 | 9 | 9 | 0 | 0 | 4 time | 4 time | 0.97× (9) | 0 |
| parallel-net | partition-table | classical net j4 | 13 | 10 | 10 | 0 | 0 | 3 time | 3 time | 0.97× (10) | 0 |
| parallel-net | partition-table | classical net j8 | 13 | 10 | 10 | 0 | 0 | 3 time | 3 time | 0.97× (10) | 0 |
| period-1 | 3-partition-mll-no | classical net j1 period 1 | 2 | 2 | 2 | 0 | 0 | – | – | 1.03× (2) | 0 |
| period-1 | partition-no | classical net j1 period 1 | 2 | 1 | 1 | 0 | 0 | 1 time | 1 time | 1.00× (1) | 0 |
| period-1 | partition-yes | classical net j1 period 1 | 1 | 1 | 1 | 0 | 0 | – | – | 1.00× (1) | 0 |
| period-1 | wide-m1 | classical net j1 period 1 | 2 | 2 | 2 | 0 | 0 | – | – | 1.02× (2) | 0 |
| period-1 | wide-m2 | classical net j1 period 1 | 2 | 2 | 2 | 0 | 0 | – | – | 1.02× (2) | 0 |
| period-16 | 3-partition-mll-no | classical net j1 period 16 | 2 | 1 | 1 | 0 | 0 | 1 time | 1 time | 0.95× (1) | 0 |
| period-16 | partition-no | classical net j1 period 16 | 2 | 0 | 0 | 0 | 0 | 2 time | 2 time | – | 0 |
| period-16 | partition-yes | classical net j1 period 16 | 1 | 0 | 0 | 0 | 0 | 1 time | 1 time | – | 0 |
| period-16 | wide-m1 | classical net j1 period 16 | 2 | 2 | 2 | 0 | 0 | – | – | 0.96× (2) | 0 |
| period-16 | wide-m2 | classical net j1 period 16 | 2 | 2 | 2 | 0 | 0 | – | – | 1.00× (2) | 0 |
| period-2 | 3-partition-mll-no | classical net j1 period 2 | 2 | 2 | 2 | 0 | 0 | – | – | 1.02× (2) | 0 |
| period-2 | partition-no | classical net j1 period 2 | 2 | 1 | 1 | 0 | 0 | 1 time | 1 time | 0.96× (1) | 0 |
| period-2 | partition-yes | classical net j1 period 2 | 1 | 1 | 1 | 0 | 0 | – | – | 0.98× (1) | 0 |
| period-2 | wide-m1 | classical net j1 period 2 | 2 | 2 | 2 | 0 | 0 | – | – | 1.02× (2) | 0 |
| period-2 | wide-m2 | classical net j1 period 2 | 2 | 2 | 2 | 0 | 0 | – | – | 1.02× (2) | 0 |
| period-8 | 3-partition-mll-no | classical net j1 period 8 | 2 | 1 | 1 | 0 | 0 | 1 time | 1 time | 0.99× (1) | 0 |
| period-8 | partition-no | classical net j1 period 8 | 2 | 0 | 0 | 0 | 0 | 2 time | 2 time | – | 0 |
| period-8 | partition-yes | classical net j1 period 8 | 1 | 0 | 0 | 0 | 0 | 1 time | 1 time | – | 0 |
| period-8 | wide-m1 | classical net j1 period 8 | 2 | 2 | 2 | 0 | 0 | – | – | 0.99× (2) | 0 |
| period-8 | wide-m2 | classical net j1 period 8 | 2 | 2 | 2 | 0 | 0 | – | – | 0.97× (2) | 0 |

### Generated and listed problems

| file | family | configuration | problem | before | after | after/before |
|---|---|---|---|--:|--:|--:|
| engines | 3-partition-mll-no | classical focus j1 | 3-partition-mll-no/4 | 15 µs ✗ | 20 µs ✗ | 1.33× |
| engines | 3-partition-mll-no | classical focus j1 | 3-partition-mll-no/5 | 18 µs ✗ | 20 µs ✗ | 1.11× |
| engines | 3-partition-mll-no | classical focus j1 | 3-partition-mll-no/6 | 17 µs ✗ | 22 µs ✗ | 1.29× |
| engines | 3-partition-mll-no | classical focus j1 | 3-partition-mll-no/8 | 16 µs ✗ | 20 µs ✗ | 1.25× |
| engines | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/4 | 1.80 s ✗ | 1.86 s ✗ | 1.04× |
| engines | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/5 | 56.31 s ✗ | 57.68 s ✗ | 1.02× |
| engines | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/6 | > 60 s | > 60 s |  |
| engines | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/8 | > 60 s | > 60 s |  |
| engines | 3-partition-mll-yes | classical focus j1 | 3-partition-mll-yes/12 | 37 µs ✓ | 42 µs ✓ | 1.14× |
| engines | 3-partition-mll-yes | classical focus j1 | 3-partition-mll-yes/4 | 31 µs ✓ | 40 µs ✓ | 1.29× |
| engines | 3-partition-mll-yes | classical focus j1 | 3-partition-mll-yes/6 | 31 µs ✓ | 42 µs ✓ | 1.35× |
| engines | 3-partition-mll-yes | classical focus j1 | 3-partition-mll-yes/8 | 33 µs ✓ | 45 µs ✓ | 1.36× |
| engines | 3-partition-mll-yes | classical net j1 | 3-partition-mll-yes/12 | > 60 s | > 60 s |  |
| engines | 3-partition-mll-yes | classical net j1 | 3-partition-mll-yes/4 | 802 µs ✓ | 806 µs ✓ | 1.00× |
| engines | 3-partition-mll-yes | classical net j1 | 3-partition-mll-yes/6 | 114.7 ms ✓ | 117.3 ms ✓ | 1.02× |
| engines | 3-partition-mll-yes | classical net j1 | 3-partition-mll-yes/8 | 5.55 s ✓ | 5.68 s ✓ | 1.02× |
| engines | additive | classical focus j1 | additive/12 | 354.7 ms ✓ | 320.2 ms ✓ | 0.90× |
| engines | additive | classical focus j1 | additive/14 | 6.19 s ✓ | 5.41 s ✓ | 0.87× |
| engines | additive | classical focus j1 | additive/16 | 12.21 s unknown | 9.95 s unknown |  |
| engines | additive | classical focus j1 | additive/8 | 1.6 ms ✓ | 1.5 ms ✓ | 0.97× |
| engines | cancellation | classical focus j1 | cancellation/3-partition-4 | 50.68 s ✓ | 319 µs ✓ | 6.3e-6× |
| engines | partition-no | classical focus j1 | partition-no/12 |  | 3.91 s ✗ |  |
| engines | partition-no | classical focus j1 | partition-no/3 | 4.2 ms ✗ | 59 µs ✗ | 1.4e-2× |
| engines | partition-no | classical focus j1 | partition-no/4 | 111.1 ms ✗ | 138 µs ✗ | 1.2e-3× |
| engines | partition-no | classical focus j1 | partition-no/9 |  | 86.1 ms ✗ |  |
| engines | partition-no | classical net j1 | partition-no/12 |  | > 60 s |  |
| engines | partition-no | classical net j1 | partition-no/3 | 3.45 s ✗ | 3.57 s ✗ | 1.04× |
| engines | partition-no | classical net j1 | partition-no/4 | > 60 s | > 60 s |  |
| engines | partition-no | classical net j1 | partition-no/9 |  | > 60 s |  |
| engines | partition-table | classical focus j1 | partition-table/1-1 | 38 µs ✓ | 33 µs ✓ | 0.87× |
| engines | partition-table | classical focus j1 | partition-table/1-1-1-1-1-7 | 58.37 s ✗ | 1.3 ms ✗ | 2.3e-5× |
| engines | partition-table | classical focus j1 | partition-table/1-1-1-5 | 61.2 ms ✗ | 149 µs ✗ | 2.4e-3× |
| engines | partition-table | classical focus j1 | partition-table/1-1-2-4 | 526 µs ✓ | 48 µs ✓ | 9.1e-2× |
| engines | partition-table | classical focus j1 | partition-table/1-1-4 | 2.3 ms ✗ | 56 µs ✗ | 2.5e-2× |
| engines | partition-table | classical focus j1 | partition-table/1-2-3-4-5-5 | 23.15 s ✓ | 80 µs ✓ | 3.5e-6× |
| engines | partition-table | classical focus j1 | partition-table/1-2-5 | 5.8 ms ✗ | 59 µs ✗ | 1.0e-2× |
| engines | partition-table | classical focus j1 | partition-table/1-3 | 131 µs ✗ | 29 µs ✗ | 0.22× |
| engines | partition-table | classical focus j1 | partition-table/2-1-1 | 165 µs ✓ | 45 µs ✓ | 0.27× |
| engines | partition-table | classical focus j1 | partition-table/2-2-1-1 | 2.2 ms ✓ | 62 µs ✓ | 2.9e-2× |
| engines | partition-table | classical focus j1 | partition-table/2-2-2-2-2-2-9-1 | > 60 s | 124 µs ✓ |  |
| engines | partition-table | classical focus j1 | partition-table/2-3-2-1 | 6.1 ms ✓ | 68 µs ✓ | 1.1e-2× |
| engines | partition-table | classical focus j1 | partition-table/3-3-3-1 | 112.2 ms ✗ | 143 µs ✗ | 1.3e-3× |
| engines | partition-table | classical net j1 | partition-table/1-1 | 36 µs ✓ | 34 µs ✓ | 0.94× |
| engines | partition-table | classical net j1 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s |  |
| engines | partition-table | classical net j1 | partition-table/1-1-1-5 | 7.50 s ✗ | 7.59 s ✗ | 1.01× |
| engines | partition-table | classical net j1 | partition-table/1-1-2-4 | 1.03 s ✓ | 1.10 s ✓ | 1.06× |
| engines | partition-table | classical net j1 | partition-table/1-1-4 | 39.9 ms ✗ | 40.7 ms ✗ | 1.02× |
| engines | partition-table | classical net j1 | partition-table/1-2-3-4-5-5 | > 60 s | > 60 s |  |
| engines | partition-table | classical net j1 | partition-table/1-2-5 | 3.66 s ✗ | 3.75 s ✗ | 1.03× |
| engines | partition-table | classical net j1 | partition-table/1-3 | 394 µs ✗ | 372 µs ✗ | 0.94× |
| engines | partition-table | classical net j1 | partition-table/2-1-1 | 657 µs ✓ | 644 µs ✓ | 0.98× |
| engines | partition-table | classical net j1 | partition-table/2-2-1-1 | 51.5 ms ✓ | 55.2 ms ✓ | 1.07× |
| engines | partition-table | classical net j1 | partition-table/2-2-2-2-2-2-9-1 | > 60 s | > 60 s |  |
| engines | partition-table | classical net j1 | partition-table/2-3-2-1 | 4.58 s ✓ | 4.67 s ✓ | 1.02× |
| engines | partition-table | classical net j1 | partition-table/3-3-3-1 | > 60 s | > 60 s |  |
| engines | partition-yes | classical focus j1 | partition-yes/12 |  | 15.2 ms ✓ |  |
| engines | partition-yes | classical focus j1 | partition-yes/20 |  | 573.7 ms ✓ |  |
| engines | partition-yes | classical focus j1 | partition-yes/4 | 2.3 ms ✓ | 60 µs ✓ | 2.6e-2× |
| engines | partition-yes | classical focus j1 | partition-yes/5 | 169.0 ms ✓ | 77 µs ✓ | 4.6e-4× |
| engines | partition-yes | classical focus j1 | partition-yes/6 | 12.15 s ✓ | 130 µs ✓ | 1.1e-5× |
| engines | partition-yes | classical net j1 | partition-yes/12 |  | > 60 s |  |
| engines | partition-yes | classical net j1 | partition-yes/20 |  | > 60 s |  |
| engines | partition-yes | classical net j1 | partition-yes/4 | 2.53 s ✓ | 2.83 s ✓ | 1.12× |
| engines | partition-yes | classical net j1 | partition-yes/5 | > 60 s | > 60 s |  |
| engines | partition-yes | classical net j1 | partition-yes/6 | > 60 s | > 60 s |  |
| engines | wide-m1 | classical focus j1 | wide-m1/16 | 765 µs ✓ | 68 µs ✓ | 8.9e-2× |
| engines | wide-m1 | classical focus j1 | wide-m1/2048 | 50.3 ms ? wide | 271.0 ms ? depth |  |
| engines | wide-m1 | classical focus j1 | wide-m1/24 | 177.5 ms ✓ | 104 µs ✓ | 5.9e-4× |
| engines | wide-m1 | classical focus j1 | wide-m1/256 | 971 µs ? wide | 5.6 ms ✓ |  |
| engines | wide-m1 | classical focus j1 | wide-m1/32 | 45.53 s ✓ | 160 µs ✓ | 3.5e-6× |
| engines | wide-m1 | classical focus j1 | wide-m1/8 | 39 µs ✓ | 43 µs ✓ | 1.10× |
| engines | wide-m1 | classical net j1 | wide-m1/16 | 111 µs ✓ | 110 µs ✓ | 0.99× |
| engines | wide-m1 | classical net j1 | wide-m1/2048 | 638.4 ms ✓ | 637.3 ms ✓ | 1.00× |
| engines | wide-m1 | classical net j1 | wide-m1/24 | 220 µs ✓ | 207 µs ✓ | 0.94× |
| engines | wide-m1 | classical net j1 | wide-m1/256 | 9.3 ms ✓ | 9.3 ms ✓ | 1.00× |
| engines | wide-m1 | classical net j1 | wide-m1/32 | 196 µs ✓ | 179 µs ✓ | 0.91× |
| engines | wide-m1 | classical net j1 | wide-m1/8 | 45 µs ✓ | 43 µs ✓ | 0.96× |
| engines | wide-m2 | classical focus j1 | wide-m2/16 | 714 µs ✓ | 66 µs ✓ | 9.2e-2× |
| engines | wide-m2 | classical focus j1 | wide-m2/2048 | 26.9 ms ? wide | 242.7 ms ? depth |  |
| engines | wide-m2 | classical focus j1 | wide-m2/24 | 171.1 ms ✓ | 97 µs ✓ | 5.7e-4× |
| engines | wide-m2 | classical focus j1 | wide-m2/256 | 566 µs ? wide | 4.3 ms ✓ |  |
| engines | wide-m2 | classical focus j1 | wide-m2/32 | 43.50 s ✓ | 141 µs ✓ | 3.2e-6× |
| engines | wide-m2 | classical focus j1 | wide-m2/8 | 38 µs ✓ | 40 µs ✓ | 1.05× |
| engines | wide-m2 | classical net j1 | wide-m2/16 | 109 µs ✓ | 103 µs ✓ | 0.94× |
| engines | wide-m2 | classical net j1 | wide-m2/2048 | 629.2 ms ✓ | 635.2 ms ✓ | 1.01× |
| engines | wide-m2 | classical net j1 | wide-m2/24 | 217 µs ✓ | 205 µs ✓ | 0.94× |
| engines | wide-m2 | classical net j1 | wide-m2/256 | 9.3 ms ✓ | 9.4 ms ✓ | 1.01× |
| engines | wide-m2 | classical net j1 | wide-m2/32 | 199 µs ✓ | 179 µs ✓ | 0.90× |
| engines | wide-m2 | classical net j1 | wide-m2/8 | 44 µs ✓ | 43 µs ✓ | 0.98× |
| engines | wide-m3 | classical focus j1 | wide-m3/12 | 84 µs ✓ | 50 µs ✓ | 0.60× |
| engines | wide-m3 | classical focus j1 | wide-m3/2048 |  | 215.8 ms ? depth |  |
| engines | wide-m3 | classical focus j1 | wide-m3/24 | 168.6 ms ✓ | 94 µs ✓ | 5.6e-4× |
| engines | wide-m3 | classical focus j1 | wide-m3/256 |  | 3.7 ms ✓ |  |
| engines | wide-m3 | classical focus j1 | wide-m3/30 | 10.85 s ✓ | 126 µs ✓ | 1.2e-5× |
| engines | wide-m3 | classical net j1 | wide-m3/12 | 71 µs ✓ | 74 µs ✓ | 1.04× |
| engines | wide-m3 | classical net j1 | wide-m3/2048 |  | 649.4 ms ✓ |  |
| engines | wide-m3 | classical net j1 | wide-m3/24 | 217 µs ✓ | 213 µs ✓ | 0.98× |
| engines | wide-m3 | classical net j1 | wide-m3/256 |  | 9.7 ms ✓ |  |
| engines | wide-m3 | classical net j1 | wide-m3/30 | 182 µs ✓ | 166 µs ✓ | 0.91× |
| engines | wide-m4 | classical focus j1 | wide-m4/12 | 87 µs ✓ | 53 µs ✓ | 0.61× |
| engines | wide-m4 | classical focus j1 | wide-m4/2048 |  | 203.7 ms ? depth |  |
| engines | wide-m4 | classical focus j1 | wide-m4/24 | 171.8 ms ✓ | 92 µs ✓ | 5.4e-4× |
| engines | wide-m4 | classical focus j1 | wide-m4/256 |  | 3.5 ms ✓ |  |
| engines | wide-m4 | classical focus j1 | wide-m4/28 | 2.75 s ✓ | 115 µs ✓ | 4.2e-5× |
| engines | wide-m4 | classical net j1 | wide-m4/12 | 76 µs ✓ | 73 µs ✓ | 0.96× |
| engines | wide-m4 | classical net j1 | wide-m4/2048 |  | 673.2 ms ✓ |  |
| engines | wide-m4 | classical net j1 | wide-m4/24 | 223 µs ✓ | 215 µs ✓ | 0.96× |
| engines | wide-m4 | classical net j1 | wide-m4/256 |  | 10.1 ms ✓ |  |
| engines | wide-m4 | classical net j1 | wide-m4/28 | 290 µs ✓ | 278 µs ✓ | 0.96× |
| families | 3-partition-mll-no | classical auto j1 | 3-partition-mll-no/4 | 14 µs ✗ | 19 µs ✗ | 1.36× |
| families | 3-partition-mll-no | classical auto j1 | 3-partition-mll-no/5 | 15 µs ✗ | 18 µs ✗ | 1.20× |
| families | 3-partition-mll-no | classical auto j1 | 3-partition-mll-no/6 | 17 µs ✗ | 20 µs ✗ | 1.18× |
| families | 3-partition-mll-no | classical auto j1 | 3-partition-mll-no/8 | 16 µs ✗ | 20 µs ✗ | 1.25× |
| families | 3-partition-mll-yes | classical auto j1 | 3-partition-mll-yes/12 | 35 µs ✓ | 41 µs ✓ | 1.17× |
| families | 3-partition-mll-yes | classical auto j1 | 3-partition-mll-yes/4 | 27 µs ✓ | 36 µs ✓ | 1.33× |
| families | 3-partition-mll-yes | classical auto j1 | 3-partition-mll-yes/6 | 28 µs ✓ | 38 µs ✓ | 1.36× |
| families | 3-partition-mll-yes | classical auto j1 | 3-partition-mll-yes/8 | 31 µs ✓ | 39 µs ✓ | 1.26× |
| families | 3-partition-no | classical auto j1 | 3-partition-no/4 | 49.77 s ✗ | 299 µs ✗ | 6.0e-6× |
| families | 3-partition-no | classical auto j1 | 3-partition-no/5 | > 300 s | 358 µs ✗ |  |
| families | 3-partition-yes | classical auto j1 | 3-partition-yes/12 | 30.1 ms ✓ | 48 µs ✓ | 1.6e-3× |
| families | 3-partition-yes | classical auto j1 | 3-partition-yes/4 | 180 µs ✓ | 41 µs ✓ | 0.23× |
| families | 3-partition-yes | classical auto j1 | 3-partition-yes/6 | 555 µs ✓ | 43 µs ✓ | 7.7e-2× |
| families | 3-partition-yes | classical auto j1 | 3-partition-yes/8 | 2.0 ms ✓ | 44 µs ✓ | 2.2e-2× |
| families | additive | classical auto j1 | additive/12 | 13.6 ms ✓ | 13.8 ms ✓ | 1.01× |
| families | additive | classical auto j1 | additive/14 | 69.7 ms ✓ | 68.0 ms ✓ | 0.98× |
| families | additive | classical auto j1 | additive/16 | 527.9 ms ✓ | 209.5 ms ✓ | 0.40× |
| families | additive | classical auto j1 | additive/8 | 363 µs ✓ | 378 µs ✓ | 1.04× |
| families | chain | classical auto j1 | chain/128 | 269.7 ms ✓ | 40.2 ms ✓ | 0.15× |
| families | chain | classical auto j1 | chain/16 | 247 µs ✓ | 151 µs ✓ | 0.61× |
| families | chain | classical auto j1 | chain/256 | 2.82 s ✓ | 321.3 ms ✓ | 0.11× |
| families | chain | classical auto j1 | chain/64 | 22.1 ms ✓ | 5.1 ms ✓ | 0.23× |
| families | counter | classical auto j1 | counter/16 | > 300 s | 248 µs ✓ |  |
| families | counter | classical auto j1 | counter/2 | 11 µs ✓ | 51 µs ✓ | 4.64× |
| families | counter | classical auto j1 | counter/32 |  | 2.18 s ✓ |  |
| families | counter | classical auto j1 | counter/4 | 84 µs ✓ | 65 µs ✓ | 0.77× |
| families | counter | classical auto j1 | counter/64 |  | 80.68 s ✓ |  |
| families | counter | classical auto j1 | counter/8 | 124.0 ms ✓ | 87 µs ✓ | 7.0e-4× |
| families | counter-over | classical auto j1 | counter-over/16 | > 300 s | 299 µs ✗ |  |
| families | counter-over | classical auto j1 | counter-over/2 | 24 µs ? bound | 55 µs ✗ |  |
| families | counter-over | classical auto j1 | counter-over/32 |  | 2.22 s ? bound |  |
| families | counter-over | classical auto j1 | counter-over/4 | 368 µs ? bound | 59 µs ✗ |  |
| families | counter-over | classical auto j1 | counter-over/64 |  | 85.03 s ? bound |  |
| families | counter-over | classical auto j1 | counter-over/8 | 492.7 ms ? bound | 110 µs ✗ |  |
| families | growing | classical auto j1 | growing/1024 | 567.8 ms ? depth | 256.8 ms ? depth |  |
| families | growing | classical auto j1 | growing/16 | 142 µs ? bound | 342 µs ? bound |  |
| families | growing | classical auto j1 | growing/256 | 79.3 ms ? bound | 41.8 ms ? bound |  |
| families | growing | classical auto j1 | growing/64 | 2.1 ms ? bound | 1.6 ms ? bound |  |
| families | mix | mix auto j1 | mix/10 | 213.99 s ✗ | 145.85 s ✗ | 0.68× |
| families | mix | mix auto j1 | mix/11 | > 300 s | > 300 s |  |
| families | mix | mix auto j1 | mix/4 | 275 µs ✗ | 302 µs ✗ | 1.10× |
| families | mix | mix auto j1 | mix/6 | 19.9 ms ✗ | 18.4 ms ✗ | 0.93× |
| families | mix | mix auto j1 | mix/8 | 1.91 s ✗ | 1.55 s ✗ | 0.81× |
| families | mix | mix auto j1 | mix/9 | 19.82 s ✗ | 14.10 s ✗ | 0.71× |
| families | partition-no | classical auto j1 | partition-no/12 |  | 3.87 s ✗ |  |
| families | partition-no | classical auto j1 | partition-no/14 |  | > 300 s |  |
| families | partition-no | classical auto j1 | partition-no/15 |  | > 300 s |  |
| families | partition-no | classical auto j1 | partition-no/3 | 4.4 ms ✗ | 59 µs ✗ | 1.3e-2× |
| families | partition-no | classical auto j1 | partition-no/4 | 113.8 ms ✗ | 141 µs ✗ | 1.2e-3× |
| families | partition-no | classical auto j1 | partition-no/5 | > 300 s | 465 µs ✗ |  |
| families | partition-no | classical auto j1 | partition-no/9 |  | 84.9 ms ✗ |  |
| families | partition-yes | classical auto j1 | partition-yes/12 |  | 14.8 ms ✓ |  |
| families | partition-yes | classical auto j1 | partition-yes/16 |  | 186.8 ms ✓ |  |
| families | partition-yes | classical auto j1 | partition-yes/20 |  | 568.4 ms ✓ |  |
| families | partition-yes | classical auto j1 | partition-yes/24 |  | 117.66 s ✓ |  |
| families | partition-yes | classical auto j1 | partition-yes/28 |  | > 300 s |  |
| families | partition-yes | classical auto j1 | partition-yes/4 | 2.3 ms ✓ | 55 µs ✓ | 2.4e-2× |
| families | partition-yes | classical auto j1 | partition-yes/5 | 186.5 ms ✓ | 76 µs ✓ | 4.1e-4× |
| families | partition-yes | classical auto j1 | partition-yes/6 | 12.74 s ✓ | 130 µs ✓ | 1.0e-5× |
| families | partition-yes | classical auto j1 | partition-yes/7 | > 300 s | 213 µs ✓ |  |
| families | qbf | classical auto j1 | qbf/12#0 | 121.0 ms ✓ | 1.8 ms ✓ | 1.5e-2× |
| families | qbf | classical auto j1 | qbf/12#1 | 28.8 ms ✗ | 502 µs ✗ | 1.7e-2× |
| families | qbf | classical auto j1 | qbf/12#2 | 45.0 ms ✓ | 925 µs ✓ | 2.1e-2× |
| families | qbf | classical auto j1 | qbf/12#3 | 19.2 ms ✗ | 371 µs ✗ | 1.9e-2× |
| families | qbf | classical auto j1 | qbf/16#0 | 7.87 s ✓ | 10.1 ms ✓ | 1.3e-3× |
| families | qbf | classical auto j1 | qbf/16#1 | 1.45 s ✗ | 1.5 ms ✗ | 1.0e-3× |
| families | qbf | classical auto j1 | qbf/16#2 | 2.09 s ✗ | 2.4 ms ✗ | 1.2e-3× |
| families | qbf | classical auto j1 | qbf/16#3 | 6.56 s ✓ | 7.7 ms ✓ | 1.2e-3× |
| families | qbf | classical auto j1 | qbf/20#0 | 87.23 s ✗ | 7.3 ms ✗ | 8.4e-5× |
| families | qbf | classical auto j1 | qbf/20#1 | 123.61 s ✗ | 12.0 ms ✗ | 9.7e-5× |
| families | qbf | classical auto j1 | qbf/20#2 | > 300 s | 49.8 ms ✓ |  |
| families | qbf | classical auto j1 | qbf/20#3 | 89.32 s ✗ | 7.1 ms ✗ | 8.0e-5× |
| families | qbf | classical auto j1 | qbf/24#0 | > 300 s | 163.5 ms ✗ |  |
| families | qbf | classical auto j1 | qbf/24#1 | > 300 s | 58.0 ms ✗ |  |
| families | qbf | classical auto j1 | qbf/24#2 | > 300 s | 38.7 ms ✗ |  |
| families | qbf | classical auto j1 | qbf/24#3 | > 300 s | 57.2 ms ✗ |  |
| families | qbf | classical auto j1 | qbf/32#0 |  | 875.9 ms ✗ |  |
| families | qbf | classical auto j1 | qbf/32#1 |  | 943.7 ms ✗ |  |
| families | qbf | classical auto j1 | qbf/32#2 |  | 2.04 s ✗ |  |
| families | qbf | classical auto j1 | qbf/32#3 |  | 778.5 ms ✗ |  |
| families | qbf | classical auto j1 | qbf/40#0 |  | 17.53 s ✗ |  |
| families | qbf | classical auto j1 | qbf/40#1 |  | 20.18 s ✗ |  |
| families | qbf | classical auto j1 | qbf/40#2 |  | 23.34 s ✗ |  |
| families | qbf | classical auto j1 | qbf/40#3 |  | 18.29 s ✗ |  |
| families | qbf | classical auto j1 | qbf/44#0 |  | 77.60 s ✗ |  |
| families | qbf | classical auto j1 | qbf/44#1 |  | 79.01 s ✗ |  |
| families | qbf | classical auto j1 | qbf/44#2 |  | 115.85 s ✗ |  |
| families | qbf | classical auto j1 | qbf/44#3 |  | 218.72 s ✗ |  |
| families | qbf | classical auto j1 | qbf/48#0 |  | 290.27 s unknown |  |
| families | qbf | classical auto j1 | qbf/48#1 |  | 282.04 s ✗ |  |
| families | qbf | classical auto j1 | qbf/48#2 |  | 230.53 s ✗ |  |
| families | qbf | classical auto j1 | qbf/48#3 |  | > 300 s |  |
| families | qbf | classical auto j1 | qbf/8#0 | 551 µs ✗ | 123 µs ✗ | 0.22× |
| families | qbf | classical auto j1 | qbf/8#1 | 587 µs ✗ | 161 µs ✗ | 0.27× |
| families | qbf | classical auto j1 | qbf/8#2 | 830 µs ✓ | 184 µs ✓ | 0.22× |
| families | qbf | classical auto j1 | qbf/8#3 | 643 µs ✗ | 141 µs ✗ | 0.22× |
| families | wide-m1 | classical auto j1 | wide-m1/16 | 110 µs ✓ | 98 µs ✓ | 0.89× |
| families | wide-m1 | classical auto j1 | wide-m1/2048 | 610.2 ms ✓ | 581.5 ms ✓ | 0.95× |
| families | wide-m1 | classical auto j1 | wide-m1/24 | 198 µs ✓ | 199 µs ✓ | 1.01× |
| families | wide-m1 | classical auto j1 | wide-m1/256 | 8.8 ms ✓ | 8.2 ms ✓ | 0.93× |
| families | wide-m1 | classical auto j1 | wide-m1/32 | 178 µs ✓ | 166 µs ✓ | 0.93× |
| families | wide-m1 | classical auto j1 | wide-m1/8 | 40 µs ✓ | 40 µs ✓ | 1.00× |
| families | wide-m2 | classical auto j1 | wide-m2/16 | 103 µs ✓ | 97 µs ✓ | 0.94× |
| families | wide-m2 | classical auto j1 | wide-m2/2048 | 602.4 ms ✓ | 571.3 ms ✓ | 0.95× |
| families | wide-m2 | classical auto j1 | wide-m2/24 | 201 µs ✓ | 191 µs ✓ | 0.95× |
| families | wide-m2 | classical auto j1 | wide-m2/256 | 8.8 ms ✓ | 8.5 ms ✓ | 0.97× |
| families | wide-m2 | classical auto j1 | wide-m2/32 | 170 µs ✓ | 165 µs ✓ | 0.97× |
| families | wide-m2 | classical auto j1 | wide-m2/8 | 40 µs ✓ | 44 µs ✓ | 1.10× |
| families | wide-m3 | classical auto j1 | wide-m3/1024 |  | 55.2 ms ✓ |  |
| families | wide-m3 | classical auto j1 | wide-m3/12 | 77 µs ✓ | 44 µs ✓ | 0.57× |
| families | wide-m3 | classical auto j1 | wide-m3/2048 |  | 204.3 ms ? depth |  |
| families | wide-m3 | classical auto j1 | wide-m3/24 | 166.4 ms ✓ | 88 µs ✓ | 5.3e-4× |
| families | wide-m3 | classical auto j1 | wide-m3/256 |  | 3.3 ms ✓ |  |
| families | wide-m3 | classical auto j1 | wide-m3/30 | 10.57 s ✓ | 112 µs ✓ | 1.1e-5× |
| families | wide-m3 | classical auto j1 | wide-m3/36 | > 300 s | 139 µs ✓ |  |
| families | wide-m4 | classical auto j1 | wide-m4/1024 |  | 51.6 ms ✓ |  |
| families | wide-m4 | classical auto j1 | wide-m4/12 | 79 µs ✓ | 49 µs ✓ | 0.62× |
| families | wide-m4 | classical auto j1 | wide-m4/2048 |  | 191.8 ms ? depth |  |
| families | wide-m4 | classical auto j1 | wide-m4/24 | 170.2 ms ✓ | 85 µs ✓ | 5.0e-4× |
| families | wide-m4 | classical auto j1 | wide-m4/256 |  | 3.2 ms ✓ |  |
| families | wide-m4 | classical auto j1 | wide-m4/28 | 2.72 s ✓ | 104 µs ✓ | 3.8e-5× |
| families | wide-m4 | classical auto j1 | wide-m4/32 | 43.39 s ✓ | 119 µs ✓ | 2.7e-6× |
| families | wide-m4 | classical auto j1 | wide-m4/36 | > 300 s | 137 µs ✓ |  |
| intuitionistic | 3-partition-no | intuitionistic auto j1 | 3-partition-no/4 | 25.38 s ✗ | 293 µs ✗ | 1.2e-5× |
| intuitionistic | 3-partition-yes | intuitionistic auto j1 | 3-partition-yes/12 | 16.5 ms ✓ | 51 µs ✓ | 3.1e-3× |
| intuitionistic | 3-partition-yes | intuitionistic auto j1 | 3-partition-yes/4 | 116 µs ✓ | 40 µs ✓ | 0.34× |
| intuitionistic | 3-partition-yes | intuitionistic auto j1 | 3-partition-yes/6 | 320 µs ✓ | 46 µs ✓ | 0.14× |
| intuitionistic | 3-partition-yes | intuitionistic auto j1 | 3-partition-yes/8 | 1.1 ms ✓ | 47 µs ✓ | 4.4e-2× |
| intuitionistic | chain | intuitionistic auto j1 | chain/128 | 275.9 ms ✓ | 42.4 ms ✓ | 0.15× |
| intuitionistic | chain | intuitionistic auto j1 | chain/16 | 273 µs ✓ | 159 µs ✓ | 0.58× |
| intuitionistic | chain | intuitionistic auto j1 | chain/256 | 2.89 s ✓ | 347.4 ms ✓ | 0.12× |
| intuitionistic | chain | intuitionistic auto j1 | chain/64 | 22.6 ms ✓ | 5.4 ms ✓ | 0.24× |
| intuitionistic | counter | intuitionistic auto j1 | counter/16 | > 300 s | 251 µs ✓ |  |
| intuitionistic | counter | intuitionistic auto j1 | counter/2 | 12 µs ✓ | 50 µs ✓ | 4.17× |
| intuitionistic | counter | intuitionistic auto j1 | counter/32 |  | 19.9 ms ✓ |  |
| intuitionistic | counter | intuitionistic auto j1 | counter/4 | 48 µs ✓ | 61 µs ✓ | 1.27× |
| intuitionistic | counter | intuitionistic auto j1 | counter/64 |  | 267.7 ms ✓ |  |
| intuitionistic | counter | intuitionistic auto j1 | counter/8 | 15.4 ms ✓ | 88 µs ✓ | 5.7e-3× |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/16 | > 300 s | 306 µs ✗ |  |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/2 | 21 µs ? bound | 54 µs ✗ |  |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/32 |  | 20.3 ms ? bound |  |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/4 | 60 µs ? bound | 66 µs ✗ |  |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/64 |  | 268.5 ms ? bound |  |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/8 | 18.4 ms ? bound | 107 µs ✗ |  |
| long-1 | 3-partition-no | classical auto j1 | 3-partition-no/5 | 302.42 s ✗ | 393 µs ✗ | 1.3e-6× |
| long-1 | counter | classical auto j1 | counter/16 | > 1200 s | 280 µs ✓ |  |
| long-1 | counter | classical auto j1 | counter/64 |  | 85.14 s ✓ |  |
| long-1 | partition-no | classical auto j1 | partition-no/15 |  | > 1200 s |  |
| long-1 | partition-no | classical auto j1 | partition-no/5 | 811.48 s ✗ | 477 µs ✗ | 5.9e-7× |
| long-1 | partition-yes | classical auto j1 | partition-yes/28 |  | > 1200 s |  |
| long-1 | partition-yes | classical auto j1 | partition-yes/7 | > 1200 s | 230 µs ✓ |  |
| long-2 | mix | mix auto j1 | mix/11 | > 1200 s | > 1200 s |  |
| long-2 | qbf | classical auto j1 | qbf/24#0 | > 1200 s | 161.2 ms ✗ |  |
| long-2 | qbf | classical auto j1 | qbf/48#0 |  | 301.01 s unknown |  |
| long-2 | wide-m3 | classical auto j1 | wide-m3/36 | 697.42 s ✓ | 157 µs ✓ | 2.3e-7× |
| long-2 | wide-m4 | classical auto j1 | wide-m4/36 | 685.07 s ✓ | 144 µs ✓ | 2.1e-7× |
| parallel | 3-partition-no | classical auto j1 | 3-partition-no/4 | 45.09 s ✗ | 395 µs ✗ | 8.8e-6× |
| parallel | 3-partition-no | classical auto j1 | 3-partition-no/5 | > 120 s | 470 µs ✗ |  |
| parallel | 3-partition-no | classical auto j16 | 3-partition-no/4 | 5.80 s ✗ | 720 µs ✗ | 1.2e-4× |
| parallel | 3-partition-no | classical auto j16 | 3-partition-no/5 | 37.07 s ✗ | 689 µs ✗ | 1.9e-5× |
| parallel | 3-partition-no | classical auto j2 | 3-partition-no/4 | 23.49 s ✗ | 439 µs ✗ | 1.9e-5× |
| parallel | 3-partition-no | classical auto j2 | 3-partition-no/5 | > 120 s | 483 µs ✗ |  |
| parallel | 3-partition-no | classical auto j4 | 3-partition-no/4 | 12.68 s ✗ | 342 µs ✗ | 2.7e-5× |
| parallel | 3-partition-no | classical auto j4 | 3-partition-no/5 | 77.70 s ✗ | 354 µs ✗ | 4.6e-6× |
| parallel | 3-partition-no | classical auto j8 | 3-partition-no/4 | 7.45 s ✗ | 467 µs ✗ | 6.3e-5× |
| parallel | 3-partition-no | classical auto j8 | 3-partition-no/5 | 46.72 s ✗ | 431 µs ✗ | 9.2e-6× |
| parallel | 3-partition-yes | classical auto j1 | 3-partition-yes/12 | 37.7 ms ✓ | 59 µs ✓ | 1.6e-3× |
| parallel | 3-partition-yes | classical auto j1 | 3-partition-yes/4 | 217 µs ✓ | 48 µs ✓ | 0.22× |
| parallel | 3-partition-yes | classical auto j1 | 3-partition-yes/6 | 674 µs ✓ | 58 µs ✓ | 8.6e-2× |
| parallel | 3-partition-yes | classical auto j1 | 3-partition-yes/8 | 2.4 ms ✓ | 54 µs ✓ | 2.2e-2× |
| parallel | 3-partition-yes | classical auto j16 | 3-partition-yes/12 | 30.4 ms ✓ | 540 µs ✓ | 1.8e-2× |
| parallel | 3-partition-yes | classical auto j16 | 3-partition-yes/4 | 706 µs ✓ | 506 µs ✓ | 0.72× |
| parallel | 3-partition-yes | classical auto j16 | 3-partition-yes/6 | 1.3 ms ✓ | 452 µs ✓ | 0.36× |
| parallel | 3-partition-yes | classical auto j16 | 3-partition-yes/8 | 2.8 ms ✓ | 601 µs ✓ | 0.22× |
| parallel | 3-partition-yes | classical auto j2 | 3-partition-yes/12 | 29.6 ms ✓ | 163 µs ✓ | 5.5e-3× |
| parallel | 3-partition-yes | classical auto j2 | 3-partition-yes/4 | 489 µs ✓ | 209 µs ✓ | 0.43× |
| parallel | 3-partition-yes | classical auto j2 | 3-partition-yes/6 | 920 µs ✓ | 142 µs ✓ | 0.15× |
| parallel | 3-partition-yes | classical auto j2 | 3-partition-yes/8 | 2.4 ms ✓ | 164 µs ✓ | 6.9e-2× |
| parallel | 3-partition-yes | classical auto j4 | 3-partition-yes/12 | 29.3 ms ✓ | 202 µs ✓ | 6.9e-3× |
| parallel | 3-partition-yes | classical auto j4 | 3-partition-yes/4 | 425 µs ✓ | 198 µs ✓ | 0.47× |
| parallel | 3-partition-yes | classical auto j4 | 3-partition-yes/6 | 911 µs ✓ | 179 µs ✓ | 0.20× |
| parallel | 3-partition-yes | classical auto j4 | 3-partition-yes/8 | 2.3 ms ✓ | 183 µs ✓ | 7.8e-2× |
| parallel | 3-partition-yes | classical auto j8 | 3-partition-yes/12 | 29.5 ms ✓ | 308 µs ✓ | 1.0e-2× |
| parallel | 3-partition-yes | classical auto j8 | 3-partition-yes/4 | 433 µs ✓ | 328 µs ✓ | 0.76× |
| parallel | 3-partition-yes | classical auto j8 | 3-partition-yes/6 | 1.0 ms ✓ | 287 µs ✓ | 0.28× |
| parallel | 3-partition-yes | classical auto j8 | 3-partition-yes/8 | 2.5 ms ✓ | 352 µs ✓ | 0.14× |
| parallel | counter | classical auto j1 | counter/16 | > 120 s | 308 µs ✓ |  |
| parallel | counter | classical auto j1 | counter/2 | 13 µs ✓ | 65 µs ✓ | 5.00× |
| parallel | counter | classical auto j1 | counter/32 |  | 2.18 s ✓ |  |
| parallel | counter | classical auto j1 | counter/4 | 110 µs ✓ | 66 µs ✓ | 0.60× |
| parallel | counter | classical auto j1 | counter/64 |  | 78.45 s ✓ |  |
| parallel | counter | classical auto j1 | counter/8 | 129.6 ms ✓ | 98 µs ✓ | 7.6e-4× |
| parallel | counter | classical auto j16 | counter/16 | > 120 s | 1.2 ms ✓ |  |
| parallel | counter | classical auto j16 | counter/2 | 479 µs ✓ | 465 µs ✓ | 0.97× |
| parallel | counter | classical auto j16 | counter/32 |  | 700.0 ms ✓ |  |
| parallel | counter | classical auto j16 | counter/4 | 645 µs ✓ | 559 µs ✓ | 0.87× |
| parallel | counter | classical auto j16 | counter/64 |  | 23.90 s ✓ |  |
| parallel | counter | classical auto j16 | counter/8 | 109.3 ms ✓ | 691 µs ✓ | 6.3e-3× |
| parallel | counter | classical auto j2 | counter/16 | > 120 s | 389 µs ✓ |  |
| parallel | counter | classical auto j2 | counter/2 | 99 µs ✓ | 109 µs ✓ | 1.10× |
| parallel | counter | classical auto j2 | counter/32 |  | 2.70 s ✓ |  |
| parallel | counter | classical auto j2 | counter/4 | 225 µs ✓ | 99 µs ✓ | 0.44× |
| parallel | counter | classical auto j2 | counter/64 |  | 101.90 s ✓ |  |
| parallel | counter | classical auto j2 | counter/8 | 156.3 ms ✓ | 138 µs ✓ | 8.8e-4× |
| parallel | counter | classical auto j4 | counter/16 | > 120 s | 648 µs ✓ |  |
| parallel | counter | classical auto j4 | counter/2 | 126 µs ✓ | 156 µs ✓ | 1.24× |
| parallel | counter | classical auto j4 | counter/32 |  | 2.02 s ✓ |  |
| parallel | counter | classical auto j4 | counter/4 | 258 µs ✓ | 161 µs ✓ | 0.62× |
| parallel | counter | classical auto j4 | counter/64 |  | 76.89 s ✓ |  |
| parallel | counter | classical auto j4 | counter/8 | 127.2 ms ✓ | 300 µs ✓ | 2.4e-3× |
| parallel | counter | classical auto j8 | counter/16 | > 120 s | 747 µs ✓ |  |
| parallel | counter | classical auto j8 | counter/2 | 253 µs ✓ | 243 µs ✓ | 0.96× |
| parallel | counter | classical auto j8 | counter/32 |  | 629.7 ms ✓ |  |
| parallel | counter | classical auto j8 | counter/4 | 321 µs ✓ | 274 µs ✓ | 0.85× |
| parallel | counter | classical auto j8 | counter/64 |  | 22.55 s ✓ |  |
| parallel | counter | classical auto j8 | counter/8 | 105.3 ms ✓ | 419 µs ✓ | 4.0e-3× |
| parallel | counter-over | classical auto j1 | counter-over/16 | > 120 s | 403 µs ✗ |  |
| parallel | counter-over | classical auto j1 | counter-over/2 | 27 µs ? bound | 65 µs ✗ |  |
| parallel | counter-over | classical auto j1 | counter-over/32 |  | 2.28 s ? bound |  |
| parallel | counter-over | classical auto j1 | counter-over/4 | 500 µs ? bound | 77 µs ✗ |  |
| parallel | counter-over | classical auto j1 | counter-over/64 |  | 82.99 s ? bound |  |
| parallel | counter-over | classical auto j1 | counter-over/8 | 549.9 ms ? bound | 116 µs ✗ |  |
| parallel | counter-over | classical auto j16 | counter-over/16 | > 120 s | 1.2 ms ✗ |  |
| parallel | counter-over | classical auto j16 | counter-over/2 | 446 µs ? bound | 510 µs ✗ |  |
| parallel | counter-over | classical auto j16 | counter-over/32 |  | 2.87 s ? bound |  |
| parallel | counter-over | classical auto j16 | counter-over/4 | 981 µs ? bound | 568 µs ✗ |  |
| parallel | counter-over | classical auto j16 | counter-over/64 |  | 103.02 s ? bound |  |
| parallel | counter-over | classical auto j16 | counter-over/8 | 323.2 ms ? bound | 759 µs ✗ |  |
| parallel | counter-over | classical auto j2 | counter-over/16 | > 120 s | 430 µs ✗ |  |
| parallel | counter-over | classical auto j2 | counter-over/2 | 102 µs ? bound | 93 µs ✗ |  |
| parallel | counter-over | classical auto j2 | counter-over/32 |  | 3.15 s ? bound |  |
| parallel | counter-over | classical auto j2 | counter-over/4 | 590 µs ? bound | 128 µs ✗ |  |
| parallel | counter-over | classical auto j2 | counter-over/64 |  | 112.73 s ? bound |  |
| parallel | counter-over | classical auto j2 | counter-over/8 | 502.6 ms ? bound | 165 µs ✗ |  |
| parallel | counter-over | classical auto j4 | counter-over/16 | > 120 s | 731 µs ✗ |  |
| parallel | counter-over | classical auto j4 | counter-over/2 | 153 µs ? bound | 150 µs ✗ |  |
| parallel | counter-over | classical auto j4 | counter-over/32 |  | 2.52 s ? bound |  |
| parallel | counter-over | classical auto j4 | counter-over/4 | 504 µs ? bound | 171 µs ✗ |  |
| parallel | counter-over | classical auto j4 | counter-over/64 |  | 86.58 s ? bound |  |
| parallel | counter-over | classical auto j4 | counter-over/8 | 389.9 ms ? bound | 274 µs ✗ |  |
| parallel | counter-over | classical auto j8 | counter-over/16 | > 120 s | 1.2 ms ✗ |  |
| parallel | counter-over | classical auto j8 | counter-over/2 | 216 µs ? bound | 240 µs ✗ |  |
| parallel | counter-over | classical auto j8 | counter-over/32 |  | 2.72 s ? bound |  |
| parallel | counter-over | classical auto j8 | counter-over/4 | 730 µs ? bound | 268 µs ✗ |  |
| parallel | counter-over | classical auto j8 | counter-over/64 |  | 95.97 s ? bound |  |
| parallel | counter-over | classical auto j8 | counter-over/8 | 355.2 ms ? bound | 454 µs ✗ |  |
| parallel | mix | mix auto j1 | mix/10 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j1 | mix/11 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j1 | mix/8 | 1.80 s ✗ | 1.56 s ✗ | 0.86× |
| parallel | mix | mix auto j1 | mix/9 | 17.95 s ✗ | 13.94 s ✗ | 0.78× |
| parallel | mix | mix auto j16 | mix/10 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j16 | mix/11 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j16 | mix/8 | 1.88 s ✗ | 2.43 s ✗ | 1.29× |
| parallel | mix | mix auto j16 | mix/9 | 16.93 s ✗ | 21.18 s ✗ | 1.25× |
| parallel | mix | mix auto j2 | mix/10 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j2 | mix/11 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j2 | mix/8 | 1.86 s ✗ | 1.87 s ✗ | 1.01× |
| parallel | mix | mix auto j2 | mix/9 | 17.97 s ✗ | 17.40 s ✗ | 0.97× |
| parallel | mix | mix auto j4 | mix/10 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j4 | mix/11 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j4 | mix/8 | 1.64 s ✗ | 1.91 s ✗ | 1.16× |
| parallel | mix | mix auto j4 | mix/9 | 16.72 s ✗ | 17.88 s ✗ | 1.07× |
| parallel | mix | mix auto j8 | mix/10 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j8 | mix/11 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j8 | mix/8 | 1.67 s ✗ | 2.07 s ✗ | 1.24× |
| parallel | mix | mix auto j8 | mix/9 | 16.01 s ✗ | 18.57 s ✗ | 1.16× |
| parallel | partition-no | classical auto j1 | partition-no/12 |  | 4.01 s ✗ |  |
| parallel | partition-no | classical auto j1 | partition-no/14 |  | > 120 s |  |
| parallel | partition-no | classical auto j1 | partition-no/4 | 127.3 ms ✗ | 170 µs ✗ | 1.3e-3× |
| parallel | partition-no | classical auto j1 | partition-no/5 | > 120 s | 604 µs ✗ |  |
| parallel | partition-no | classical auto j16 | partition-no/12 |  | 847.6 ms ✗ |  |
| parallel | partition-no | classical auto j16 | partition-no/14 |  | > 120 s |  |
| parallel | partition-no | classical auto j16 | partition-no/4 | 26.9 ms ✗ | 721 µs ✗ | 2.7e-2× |
| parallel | partition-no | classical auto j16 | partition-no/5 | > 120 s | 840 µs ✗ |  |
| parallel | partition-no | classical auto j2 | partition-no/12 |  | 2.83 s ✗ |  |
| parallel | partition-no | classical auto j2 | partition-no/14 |  | > 120 s |  |
| parallel | partition-no | classical auto j2 | partition-no/4 | 73.9 ms ✗ | 272 µs ✗ | 3.7e-3× |
| parallel | partition-no | classical auto j2 | partition-no/5 | > 120 s | 681 µs ✗ |  |
| parallel | partition-no | classical auto j4 | partition-no/12 |  | 1.73 s ✗ |  |
| parallel | partition-no | classical auto j4 | partition-no/14 |  | > 120 s |  |
| parallel | partition-no | classical auto j4 | partition-no/4 | 45.9 ms ✗ | 242 µs ✗ | 5.3e-3× |
| parallel | partition-no | classical auto j4 | partition-no/5 | > 120 s | 518 µs ✗ |  |
| parallel | partition-no | classical auto j8 | partition-no/12 |  | 1.20 s ✗ |  |
| parallel | partition-no | classical auto j8 | partition-no/14 |  | > 120 s |  |
| parallel | partition-no | classical auto j8 | partition-no/4 | 29.4 ms ✗ | 392 µs ✗ | 1.3e-2× |
| parallel | partition-no | classical auto j8 | partition-no/5 | > 120 s | 501 µs ✗ |  |
| parallel | partition-yes | classical auto j1 | partition-yes/20 |  | 535.5 ms ✓ |  |
| parallel | partition-yes | classical auto j1 | partition-yes/24 |  | 105.53 s ✓ |  |
| parallel | partition-yes | classical auto j1 | partition-yes/5 | 191.6 ms ✓ | 97 µs ✓ | 5.1e-4× |
| parallel | partition-yes | classical auto j1 | partition-yes/6 | 12.20 s ✓ | 167 µs ✓ | 1.4e-5× |
| parallel | partition-yes | classical auto j1 | partition-yes/7 | > 120 s | 318 µs ✓ |  |
| parallel | partition-yes | classical auto j16 | partition-yes/20 |  | 651.0 ms ✓ |  |
| parallel | partition-yes | classical auto j16 | partition-yes/24 |  | > 120 s |  |
| parallel | partition-yes | classical auto j16 | partition-yes/5 | 17.5 ms ✓ | 703 µs ✓ | 4.0e-2× |
| parallel | partition-yes | classical auto j16 | partition-yes/6 | 1.33 s ✓ | 743 µs ✓ | 5.6e-4× |
| parallel | partition-yes | classical auto j16 | partition-yes/7 | > 120 s | 849 µs ✓ |  |
| parallel | partition-yes | classical auto j2 | partition-yes/20 |  | 586.4 ms ✓ |  |
| parallel | partition-yes | classical auto j2 | partition-yes/24 |  | > 120 s |  |
| parallel | partition-yes | classical auto j2 | partition-yes/5 | 192.0 ms ✓ | 207 µs ✓ | 1.1e-3× |
| parallel | partition-yes | classical auto j2 | partition-yes/6 | 11.82 s ✓ | 280 µs ✓ | 2.4e-5× |
| parallel | partition-yes | classical auto j2 | partition-yes/7 | > 120 s | 447 µs ✓ |  |
| parallel | partition-yes | classical auto j4 | partition-yes/20 |  | 608.0 ms ✓ |  |
| parallel | partition-yes | classical auto j4 | partition-yes/24 |  | > 120 s |  |
| parallel | partition-yes | classical auto j4 | partition-yes/5 | 195.6 ms ✓ | 260 µs ✓ | 1.3e-3× |
| parallel | partition-yes | classical auto j4 | partition-yes/6 | 8.70 s ✓ | 287 µs ✓ | 3.3e-5× |
| parallel | partition-yes | classical auto j4 | partition-yes/7 | > 120 s | 498 µs ✓ |  |
| parallel | partition-yes | classical auto j8 | partition-yes/20 |  | 634.9 ms ✓ |  |
| parallel | partition-yes | classical auto j8 | partition-yes/24 |  | > 120 s |  |
| parallel | partition-yes | classical auto j8 | partition-yes/5 | 30.5 ms ✓ | 399 µs ✓ | 1.3e-2× |
| parallel | partition-yes | classical auto j8 | partition-yes/6 | 4.21 s ✓ | 450 µs ✓ | 1.1e-4× |
| parallel | partition-yes | classical auto j8 | partition-yes/7 | > 120 s | 638 µs ✓ |  |
| parallel | qbf | classical auto j1 | qbf/16#0 | 8.28 s ✓ | 13.1 ms ✓ | 1.6e-3× |
| parallel | qbf | classical auto j1 | qbf/16#1 | 1.34 s ✗ | 1.9 ms ✗ | 1.4e-3× |
| parallel | qbf | classical auto j1 | qbf/16#2 | 1.89 s ✗ | 3.3 ms ✗ | 1.7e-3× |
| parallel | qbf | classical auto j1 | qbf/16#3 | 5.99 s ✓ | 10.3 ms ✓ | 1.7e-3× |
| parallel | qbf | classical auto j1 | qbf/20#0 | 78.42 s ✗ | 9.4 ms ✗ | 1.2e-4× |
| parallel | qbf | classical auto j1 | qbf/20#1 | 109.45 s ✗ | 16.1 ms ✗ | 1.5e-4× |
| parallel | qbf | classical auto j1 | qbf/20#2 | > 120 s | 60.2 ms ✓ |  |
| parallel | qbf | classical auto j1 | qbf/20#3 | 81.26 s ✗ | 9.2 ms ✗ | 1.1e-4× |
| parallel | qbf | classical auto j1 | qbf/24#0 | > 120 s | 169.3 ms ✗ |  |
| parallel | qbf | classical auto j1 | qbf/24#1 | > 120 s | 62.6 ms ✗ |  |
| parallel | qbf | classical auto j1 | qbf/24#2 | > 120 s | 44.7 ms ✗ |  |
| parallel | qbf | classical auto j1 | qbf/24#3 | > 120 s | 62.2 ms ✗ |  |
| parallel | qbf | classical auto j1 | qbf/40#0 |  | 16.61 s ✗ |  |
| parallel | qbf | classical auto j1 | qbf/40#1 |  | 18.54 s ✗ |  |
| parallel | qbf | classical auto j1 | qbf/40#2 |  | 21.36 s ✗ |  |
| parallel | qbf | classical auto j1 | qbf/40#3 |  | 16.91 s ✗ |  |
| parallel | qbf | classical auto j1 | qbf/44#0 |  | 72.46 s ✗ |  |
| parallel | qbf | classical auto j1 | qbf/44#1 |  | 72.68 s ✗ |  |
| parallel | qbf | classical auto j1 | qbf/44#2 |  | 108.46 s ✗ |  |
| parallel | qbf | classical auto j1 | qbf/44#3 |  | > 120 s |  |
| parallel | qbf | classical auto j16 | qbf/16#0 | 6.33 s ✓ | 14.9 ms ✓ | 2.4e-3× |
| parallel | qbf | classical auto j16 | qbf/16#1 | 709.2 ms ✗ | 2.4 ms ✗ | 3.4e-3× |
| parallel | qbf | classical auto j16 | qbf/16#2 | 1.19 s ✗ | 3.4 ms ✗ | 2.9e-3× |
| parallel | qbf | classical auto j16 | qbf/16#3 | 5.50 s ✓ | 13.2 ms ✓ | 2.4e-3× |
| parallel | qbf | classical auto j16 | qbf/20#0 | 43.42 s ✗ | 7.8 ms ✗ | 1.8e-4× |
| parallel | qbf | classical auto j16 | qbf/20#1 | 61.09 s ✗ | 12.4 ms ✗ | 2.0e-4× |
| parallel | qbf | classical auto j16 | qbf/20#2 | > 120 s | 103.9 ms ✓ |  |
| parallel | qbf | classical auto j16 | qbf/20#3 | 41.57 s ✗ | 7.5 ms ✗ | 1.8e-4× |
| parallel | qbf | classical auto j16 | qbf/24#0 | > 120 s | 214.4 ms ✗ |  |
| parallel | qbf | classical auto j16 | qbf/24#1 | > 120 s | 60.5 ms ✗ |  |
| parallel | qbf | classical auto j16 | qbf/24#2 | > 120 s | 40.2 ms ✗ |  |
| parallel | qbf | classical auto j16 | qbf/24#3 | > 120 s | 58.9 ms ✗ |  |
| parallel | qbf | classical auto j16 | qbf/40#0 |  | 14.33 s ✗ |  |
| parallel | qbf | classical auto j16 | qbf/40#1 |  | 15.24 s ✗ |  |
| parallel | qbf | classical auto j16 | qbf/40#2 |  | 17.72 s ✗ |  |
| parallel | qbf | classical auto j16 | qbf/40#3 |  | 15.47 s ✗ |  |
| parallel | qbf | classical auto j16 | qbf/44#0 |  | 54.98 s ✗ |  |
| parallel | qbf | classical auto j16 | qbf/44#1 |  | 59.98 s ✗ |  |
| parallel | qbf | classical auto j16 | qbf/44#2 |  | 96.80 s ✗ |  |
| parallel | qbf | classical auto j16 | qbf/44#3 |  | > 120 s |  |
| parallel | qbf | classical auto j2 | qbf/16#0 | 6.37 s ✓ | 14.7 ms ✓ | 2.3e-3× |
| parallel | qbf | classical auto j2 | qbf/16#1 | 707.1 ms ✗ | 2.1 ms ✗ | 2.9e-3× |
| parallel | qbf | classical auto j2 | qbf/16#2 | 1.21 s ✗ | 3.6 ms ✗ | 3.0e-3× |
| parallel | qbf | classical auto j2 | qbf/16#3 | 5.53 s ✓ | 12.3 ms ✓ | 2.2e-3× |
| parallel | qbf | classical auto j2 | qbf/20#0 | 43.03 s ✗ | 9.0 ms ✗ | 2.1e-4× |
| parallel | qbf | classical auto j2 | qbf/20#1 | 61.36 s ✗ | 14.7 ms ✗ | 2.4e-4× |
| parallel | qbf | classical auto j2 | qbf/20#2 | > 120 s | 103.0 ms ✓ |  |
| parallel | qbf | classical auto j2 | qbf/20#3 | 40.98 s ✗ | 7.7 ms ✗ | 1.9e-4× |
| parallel | qbf | classical auto j2 | qbf/24#0 | > 120 s | 201.0 ms ✗ |  |
| parallel | qbf | classical auto j2 | qbf/24#1 | > 120 s | 58.6 ms ✗ |  |
| parallel | qbf | classical auto j2 | qbf/24#2 | > 120 s | 43.7 ms ✗ |  |
| parallel | qbf | classical auto j2 | qbf/24#3 | > 120 s | 58.2 ms ✗ |  |
| parallel | qbf | classical auto j2 | qbf/40#0 |  | 14.07 s ✗ |  |
| parallel | qbf | classical auto j2 | qbf/40#1 |  | 15.30 s ✗ |  |
| parallel | qbf | classical auto j2 | qbf/40#2 |  | 17.69 s ✗ |  |
| parallel | qbf | classical auto j2 | qbf/40#3 |  | 15.63 s ✗ |  |
| parallel | qbf | classical auto j2 | qbf/44#0 |  | 54.94 s ✗ |  |
| parallel | qbf | classical auto j2 | qbf/44#1 |  | 60.47 s ✗ |  |
| parallel | qbf | classical auto j2 | qbf/44#2 |  | 97.70 s ✗ |  |
| parallel | qbf | classical auto j2 | qbf/44#3 |  | > 120 s |  |
| parallel | qbf | classical auto j4 | qbf/16#0 | 6.26 s ✓ | 14.3 ms ✓ | 2.3e-3× |
| parallel | qbf | classical auto j4 | qbf/16#1 | 709.6 ms ✗ | 1.7 ms ✗ | 2.4e-3× |
| parallel | qbf | classical auto j4 | qbf/16#2 | 1.19 s ✗ | 3.7 ms ✗ | 3.1e-3× |
| parallel | qbf | classical auto j4 | qbf/16#3 | 5.52 s ✓ | 12.3 ms ✓ | 2.2e-3× |
| parallel | qbf | classical auto j4 | qbf/20#0 | 43.23 s ✗ | 8.7 ms ✗ | 2.0e-4× |
| parallel | qbf | classical auto j4 | qbf/20#1 | 61.74 s ✗ | 15.0 ms ✗ | 2.4e-4× |
| parallel | qbf | classical auto j4 | qbf/20#2 | > 120 s | 100.4 ms ✓ |  |
| parallel | qbf | classical auto j4 | qbf/20#3 | 41.17 s ✗ | 8.0 ms ✗ | 1.9e-4× |
| parallel | qbf | classical auto j4 | qbf/24#0 | > 120 s | 194.5 ms ✗ |  |
| parallel | qbf | classical auto j4 | qbf/24#1 | > 120 s | 59.7 ms ✗ |  |
| parallel | qbf | classical auto j4 | qbf/24#2 | > 120 s | 37.6 ms ✗ |  |
| parallel | qbf | classical auto j4 | qbf/24#3 | > 120 s | 58.6 ms ✗ |  |
| parallel | qbf | classical auto j4 | qbf/40#0 |  | 13.95 s ✗ |  |
| parallel | qbf | classical auto j4 | qbf/40#1 |  | 15.25 s ✗ |  |
| parallel | qbf | classical auto j4 | qbf/40#2 |  | 17.64 s ✗ |  |
| parallel | qbf | classical auto j4 | qbf/40#3 |  | 15.46 s ✗ |  |
| parallel | qbf | classical auto j4 | qbf/44#0 |  | 55.96 s ✗ |  |
| parallel | qbf | classical auto j4 | qbf/44#1 |  | 60.23 s ✗ |  |
| parallel | qbf | classical auto j4 | qbf/44#2 |  | 97.99 s ✗ |  |
| parallel | qbf | classical auto j4 | qbf/44#3 |  | > 120 s |  |
| parallel | qbf | classical auto j8 | qbf/16#0 | 6.32 s ✓ | 14.0 ms ✓ | 2.2e-3× |
| parallel | qbf | classical auto j8 | qbf/16#1 | 713.0 ms ✗ | 1.9 ms ✗ | 2.6e-3× |
| parallel | qbf | classical auto j8 | qbf/16#2 | 1.21 s ✗ | 3.3 ms ✗ | 2.7e-3× |
| parallel | qbf | classical auto j8 | qbf/16#3 | 5.76 s ✓ | 12.6 ms ✓ | 2.2e-3× |
| parallel | qbf | classical auto j8 | qbf/20#0 | 43.89 s ✗ | 8.6 ms ✗ | 2.0e-4× |
| parallel | qbf | classical auto j8 | qbf/20#1 | 61.97 s ✗ | 13.8 ms ✗ | 2.2e-4× |
| parallel | qbf | classical auto j8 | qbf/20#2 | > 120 s | 100.1 ms ✓ |  |
| parallel | qbf | classical auto j8 | qbf/20#3 | 55.61 s ✗ | 7.0 ms ✗ | 1.3e-4× |
| parallel | qbf | classical auto j8 | qbf/24#0 | > 120 s | 196.9 ms ✗ |  |
| parallel | qbf | classical auto j8 | qbf/24#1 | > 120 s | 58.5 ms ✗ |  |
| parallel | qbf | classical auto j8 | qbf/24#2 | > 120 s | 39.0 ms ✗ |  |
| parallel | qbf | classical auto j8 | qbf/24#3 | > 120 s | 58.8 ms ✗ |  |
| parallel | qbf | classical auto j8 | qbf/40#0 |  | 13.93 s ✗ |  |
| parallel | qbf | classical auto j8 | qbf/40#1 |  | 15.32 s ✗ |  |
| parallel | qbf | classical auto j8 | qbf/40#2 |  | 17.64 s ✗ |  |
| parallel | qbf | classical auto j8 | qbf/40#3 |  | 15.53 s ✗ |  |
| parallel | qbf | classical auto j8 | qbf/44#0 |  | 55.39 s ✗ |  |
| parallel | qbf | classical auto j8 | qbf/44#1 |  | 59.90 s ✗ |  |
| parallel | qbf | classical auto j8 | qbf/44#2 |  | 97.53 s ✗ |  |
| parallel | qbf | classical auto j8 | qbf/44#3 |  | > 120 s |  |
| parallel | wide-m3 | classical auto j1 | wide-m3/24 | 189.7 ms ✓ | 115 µs ✓ | 6.1e-4× |
| parallel | wide-m3 | classical auto j1 | wide-m3/30 | 11.12 s ✓ | 152 µs ✓ | 1.4e-5× |
| parallel | wide-m3 | classical auto j1 | wide-m3/36 | > 120 s | 182 µs ✓ |  |
| parallel | wide-m3 | classical auto j16 | wide-m3/24 | 131.0 ms ✓ | 666 µs ✓ | 5.1e-3× |
| parallel | wide-m3 | classical auto j16 | wide-m3/30 | 8.24 s ✓ | 611 µs ✓ | 7.4e-5× |
| parallel | wide-m3 | classical auto j16 | wide-m3/36 | > 120 s | 687 µs ✓ |  |
| parallel | wide-m3 | classical auto j2 | wide-m3/24 | 191.7 ms ✓ | 198 µs ✓ | 1.0e-3× |
| parallel | wide-m3 | classical auto j2 | wide-m3/30 | 10.69 s ✓ | 269 µs ✓ | 2.5e-5× |
| parallel | wide-m3 | classical auto j2 | wide-m3/36 | > 120 s | 274 µs ✓ |  |
| parallel | wide-m3 | classical auto j4 | wide-m3/24 | 149.5 ms ✓ | 254 µs ✓ | 1.7e-3× |
| parallel | wide-m3 | classical auto j4 | wide-m3/30 | 8.34 s ✓ | 306 µs ✓ | 3.7e-5× |
| parallel | wide-m3 | classical auto j4 | wide-m3/36 | > 120 s | 318 µs ✓ |  |
| parallel | wide-m3 | classical auto j8 | wide-m3/24 | 138.8 ms ✓ | 331 µs ✓ | 2.4e-3× |
| parallel | wide-m3 | classical auto j8 | wide-m3/30 | 7.87 s ✓ | 424 µs ✓ | 5.4e-5× |
| parallel | wide-m3 | classical auto j8 | wide-m3/36 | > 120 s | 424 µs ✓ |  |
| parallel | wide-m4 | classical auto j1 | wide-m4/28 | 2.97 s ✓ | 149 µs ✓ | 5.0e-5× |
| parallel | wide-m4 | classical auto j1 | wide-m4/32 | 42.28 s ✓ | 179 µs ✓ | 4.2e-6× |
| parallel | wide-m4 | classical auto j1 | wide-m4/36 | > 120 s | 175 µs ✓ |  |
| parallel | wide-m4 | classical auto j16 | wide-m4/28 | 1.98 s ✓ | 699 µs ✓ | 3.5e-4× |
| parallel | wide-m4 | classical auto j16 | wide-m4/32 | 35.49 s ✓ | 620 µs ✓ | 1.7e-5× |
| parallel | wide-m4 | classical auto j16 | wide-m4/36 | > 120 s | 638 µs ✓ |  |
| parallel | wide-m4 | classical auto j2 | wide-m4/28 | 2.92 s ✓ | 229 µs ✓ | 7.8e-5× |
| parallel | wide-m4 | classical auto j2 | wide-m4/32 | 41.95 s ✓ | 266 µs ✓ | 6.3e-6× |
| parallel | wide-m4 | classical auto j2 | wide-m4/36 | > 120 s | 270 µs ✓ |  |
| parallel | wide-m4 | classical auto j4 | wide-m4/28 | 2.19 s ✓ | 267 µs ✓ | 1.2e-4× |
| parallel | wide-m4 | classical auto j4 | wide-m4/32 | 32.67 s ✓ | 284 µs ✓ | 8.7e-6× |
| parallel | wide-m4 | classical auto j4 | wide-m4/36 | > 120 s | 329 µs ✓ |  |
| parallel | wide-m4 | classical auto j8 | wide-m4/28 | 2.12 s ✓ | 369 µs ✓ | 1.7e-4× |
| parallel | wide-m4 | classical auto j8 | wide-m4/32 | 32.75 s ✓ | 393 µs ✓ | 1.2e-5× |
| parallel | wide-m4 | classical auto j8 | wide-m4/36 | > 120 s | 412 µs ✓ |  |
| parallel-net | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/4 | 1.59 s ✗ | 1.70 s ✗ | 1.07× |
| parallel-net | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/5 | 59.71 s ✗ | 51.56 s ✗ | 0.86× |
| parallel-net | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/6 | > 60 s | > 60 s |  |
| parallel-net | 3-partition-mll-no | classical net j16 | 3-partition-mll-no/4 | 142.0 ms ✗ | 139.7 ms ✗ | 0.98× |
| parallel-net | 3-partition-mll-no | classical net j16 | 3-partition-mll-no/5 | 4.60 s ✗ | 4.51 s ✗ | 0.98× |
| parallel-net | 3-partition-mll-no | classical net j16 | 3-partition-mll-no/6 | > 60 s | > 60 s |  |
| parallel-net | 3-partition-mll-no | classical net j2 | 3-partition-mll-no/4 | 820.1 ms ✗ | 886.3 ms ✗ | 1.08× |
| parallel-net | 3-partition-mll-no | classical net j2 | 3-partition-mll-no/5 | 26.06 s ✗ | 26.63 s ✗ | 1.02× |
| parallel-net | 3-partition-mll-no | classical net j2 | 3-partition-mll-no/6 | > 60 s | > 60 s |  |
| parallel-net | 3-partition-mll-no | classical net j4 | 3-partition-mll-no/4 | 437.4 ms ✗ | 463.9 ms ✗ | 1.06× |
| parallel-net | 3-partition-mll-no | classical net j4 | 3-partition-mll-no/5 | 13.98 s ✗ | 14.46 s ✗ | 1.03× |
| parallel-net | 3-partition-mll-no | classical net j4 | 3-partition-mll-no/6 | > 60 s | > 60 s |  |
| parallel-net | 3-partition-mll-no | classical net j8 | 3-partition-mll-no/4 | 249.9 ms ✗ | 254.6 ms ✗ | 1.02× |
| parallel-net | 3-partition-mll-no | classical net j8 | 3-partition-mll-no/5 | 8.01 s ✗ | 7.92 s ✗ | 0.99× |
| parallel-net | 3-partition-mll-no | classical net j8 | 3-partition-mll-no/6 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j1 | partition-table/1-1 | 39 µs ✓ | 40 µs ✓ | 1.03× |
| parallel-net | partition-table | classical net j1 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j1 | partition-table/1-1-1-5 | 7.40 s ✗ | 7.11 s ✗ | 0.96× |
| parallel-net | partition-table | classical net j1 | partition-table/1-1-2-4 | 1.07 s ✓ | 1.04 s ✓ | 0.97× |
| parallel-net | partition-table | classical net j1 | partition-table/1-1-4 | 51.5 ms ✗ | 47.5 ms ✗ | 0.92× |
| parallel-net | partition-table | classical net j1 | partition-table/1-2-3-4-5-5 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j1 | partition-table/1-2-5 | 4.09 s ✗ | 3.92 s ✗ | 0.96× |
| parallel-net | partition-table | classical net j1 | partition-table/1-3 | 478 µs ✗ | 472 µs ✗ | 0.99× |
| parallel-net | partition-table | classical net j1 | partition-table/2-1-1 | 782 µs ✓ | 797 µs ✓ | 1.02× |
| parallel-net | partition-table | classical net j1 | partition-table/2-2-1-1 | 61.9 ms ✓ | 60.3 ms ✓ | 0.98× |
| parallel-net | partition-table | classical net j1 | partition-table/2-2-2-2-2-2-9-1 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j1 | partition-table/2-3-2-1 | 4.34 s ✓ | 4.38 s ✓ | 1.01× |
| parallel-net | partition-table | classical net j1 | partition-table/3-3-3-1 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j16 | partition-table/1-1 | 482 µs ✓ | 505 µs ✓ | 1.05× |
| parallel-net | partition-table | classical net j16 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j16 | partition-table/1-1-1-5 | 611.3 ms ✗ | 598.5 ms ✗ | 0.98× |
| parallel-net | partition-table | classical net j16 | partition-table/1-1-2-4 | 76.5 ms ✓ | 72.5 ms ✓ | 0.95× |
| parallel-net | partition-table | classical net j16 | partition-table/1-1-4 | 5.2 ms ✗ | 5.3 ms ✗ | 1.01× |
| parallel-net | partition-table | classical net j16 | partition-table/1-2-3-4-5-5 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j16 | partition-table/1-2-5 | 309.5 ms ✗ | 297.9 ms ✗ | 0.96× |
| parallel-net | partition-table | classical net j16 | partition-table/1-3 | 1.6 ms ✗ | 1.6 ms ✗ | 1.00× |
| parallel-net | partition-table | classical net j16 | partition-table/2-1-1 | 4.4 ms ✓ | 4.4 ms ✓ | 1.00× |
| parallel-net | partition-table | classical net j16 | partition-table/2-2-1-1 | 7.7 ms ✓ | 7.1 ms ✓ | 0.92× |
| parallel-net | partition-table | classical net j16 | partition-table/2-2-2-2-2-2-9-1 | 955.4 ms ✓ | 835.8 ms ✓ | 0.87× |
| parallel-net | partition-table | classical net j16 | partition-table/2-3-2-1 | 337.5 ms ✓ | 316.1 ms ✓ | 0.94× |
| parallel-net | partition-table | classical net j16 | partition-table/3-3-3-1 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j2 | partition-table/1-1 | 203 µs ✓ | 245 µs ✓ | 1.21× |
| parallel-net | partition-table | classical net j2 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j2 | partition-table/1-1-1-5 | 3.91 s ✗ | 3.63 s ✗ | 0.93× |
| parallel-net | partition-table | classical net j2 | partition-table/1-1-2-4 | 554.8 ms ✓ | 522.5 ms ✓ | 0.94× |
| parallel-net | partition-table | classical net j2 | partition-table/1-1-4 | 25.8 ms ✗ | 25.6 ms ✗ | 0.99× |
| parallel-net | partition-table | classical net j2 | partition-table/1-2-3-4-5-5 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j2 | partition-table/1-2-5 | 2.04 s ✗ | 1.92 s ✗ | 0.94× |
| parallel-net | partition-table | classical net j2 | partition-table/1-3 | 557 µs ✗ | 554 µs ✗ | 0.99× |
| parallel-net | partition-table | classical net j2 | partition-table/2-1-1 | 930 µs ✓ | 931 µs ✓ | 1.00× |
| parallel-net | partition-table | classical net j2 | partition-table/2-2-1-1 | 31.6 ms ✓ | 30.7 ms ✓ | 0.97× |
| parallel-net | partition-table | classical net j2 | partition-table/2-2-2-2-2-2-9-1 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j2 | partition-table/2-3-2-1 | 2.27 s ✓ | 2.13 s ✓ | 0.94× |
| parallel-net | partition-table | classical net j2 | partition-table/3-3-3-1 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j4 | partition-table/1-1 | 239 µs ✓ | 253 µs ✓ | 1.06× |
| parallel-net | partition-table | classical net j4 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j4 | partition-table/1-1-1-5 | 2.02 s ✗ | 1.92 s ✗ | 0.95× |
| parallel-net | partition-table | classical net j4 | partition-table/1-1-2-4 | 281.9 ms ✓ | 265.7 ms ✓ | 0.94× |
| parallel-net | partition-table | classical net j4 | partition-table/1-1-4 | 13.5 ms ✗ | 13.3 ms ✗ | 0.99× |
| parallel-net | partition-table | classical net j4 | partition-table/1-2-3-4-5-5 | 3.5 ms ✓ | 3.4 ms ✓ | 0.97× |
| parallel-net | partition-table | classical net j4 | partition-table/1-2-5 | 1.02 s ✗ | 962.7 ms ✗ | 0.95× |
| parallel-net | partition-table | classical net j4 | partition-table/1-3 | 1.3 ms ✗ | 1.3 ms ✗ | 1.01× |
| parallel-net | partition-table | classical net j4 | partition-table/2-1-1 | 4.1 ms ✓ | 4.2 ms ✓ | 1.02× |
| parallel-net | partition-table | classical net j4 | partition-table/2-2-1-1 | 17.3 ms ✓ | 16.7 ms ✓ | 0.97× |
| parallel-net | partition-table | classical net j4 | partition-table/2-2-2-2-2-2-9-1 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j4 | partition-table/2-3-2-1 | 1.19 s ✓ | 1.06 s ✓ | 0.89× |
| parallel-net | partition-table | classical net j4 | partition-table/3-3-3-1 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j8 | partition-table/1-1 | 384 µs ✓ | 312 µs ✓ | 0.81× |
| parallel-net | partition-table | classical net j8 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j8 | partition-table/1-1-1-5 | 1.11 s ✗ | 1.08 s ✗ | 0.97× |
| parallel-net | partition-table | classical net j8 | partition-table/1-1-2-4 | 149.5 ms ✓ | 143.8 ms ✓ | 0.96× |
| parallel-net | partition-table | classical net j8 | partition-table/1-1-4 | 8.4 ms ✗ | 7.9 ms ✗ | 0.94× |
| parallel-net | partition-table | classical net j8 | partition-table/1-2-3-4-5-5 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j8 | partition-table/1-2-5 | 582.3 ms ✗ | 545.7 ms ✗ | 0.94× |
| parallel-net | partition-table | classical net j8 | partition-table/1-3 | 1.4 ms ✗ | 1.4 ms ✗ | 1.01× |
| parallel-net | partition-table | classical net j8 | partition-table/2-1-1 | 4.3 ms ✓ | 4.3 ms ✓ | 1.01× |
| parallel-net | partition-table | classical net j8 | partition-table/2-2-1-1 | 9.3 ms ✓ | 9.4 ms ✓ | 1.01× |
| parallel-net | partition-table | classical net j8 | partition-table/2-2-2-2-2-2-9-1 | 861.2 ms ✓ | 897.2 ms ✓ | 1.04× |
| parallel-net | partition-table | classical net j8 | partition-table/2-3-2-1 | 654.3 ms ✓ | 611.1 ms ✓ | 0.93× |
| parallel-net | partition-table | classical net j8 | partition-table/3-3-3-1 | > 60 s | > 60 s |  |
| period-1 | 3-partition-mll-no | classical net j1 period 1 | 3-partition-mll-no/4 | 1.80 s ✗ | 1.85 s ✗ | 1.03× |
| period-1 | 3-partition-mll-no | classical net j1 period 1 | 3-partition-mll-no/5 | 56.27 s ✗ | 56.38 s ✗ | 1.00× |
| period-1 | partition-no | classical net j1 period 1 | partition-no/3 | 3.49 s ✗ | 3.49 s ✗ | 1.00× |
| period-1 | partition-no | classical net j1 period 1 | partition-no/4 | > 60 s | > 60 s |  |
| period-1 | partition-yes | classical net j1 period 1 | partition-yes/4 | 2.76 s ✓ | 2.75 s ✓ | 1.00× |
| period-1 | wide-m1 | classical net j1 period 1 | wide-m1/2048 | 1.35 s ✓ | 1.35 s ✓ | 1.00× |
| period-1 | wide-m1 | classical net j1 period 1 | wide-m1/256 | 19.2 ms ✓ | 19.5 ms ✓ | 1.02× |
| period-1 | wide-m2 | classical net j1 period 1 | wide-m2/2048 | 1.35 s ✓ | 1.38 s ✓ | 1.02× |
| period-1 | wide-m2 | classical net j1 period 1 | wide-m2/256 | 19.3 ms ✓ | 19.6 ms ✓ | 1.01× |
| period-16 | 3-partition-mll-no | classical net j1 period 16 | 3-partition-mll-no/4 | 22.04 s ✗ | 20.98 s ✗ | 0.95× |
| period-16 | 3-partition-mll-no | classical net j1 period 16 | 3-partition-mll-no/5 | > 60 s | > 60 s |  |
| period-16 | partition-no | classical net j1 period 16 | partition-no/3 | > 60 s | > 60 s |  |
| period-16 | partition-no | classical net j1 period 16 | partition-no/4 | > 60 s | > 60 s |  |
| period-16 | partition-yes | classical net j1 period 16 | partition-yes/4 | > 60 s | > 60 s |  |
| period-16 | wide-m1 | classical net j1 period 16 | wide-m1/2048 | 459.5 ms ✓ | 434.9 ms ✓ | 0.95× |
| period-16 | wide-m1 | classical net j1 period 16 | wide-m1/256 | 6.5 ms ✓ | 6.2 ms ✓ | 0.96× |
| period-16 | wide-m2 | classical net j1 period 16 | wide-m2/2048 | 442.5 ms ✓ | 429.6 ms ✓ | 0.97× |
| period-16 | wide-m2 | classical net j1 period 16 | wide-m2/256 | 6.6 ms ✓ | 6.6 ms ✓ | 1.00× |
| period-2 | 3-partition-mll-no | classical net j1 period 2 | 3-partition-mll-no/4 | 1.43 s ✗ | 1.45 s ✗ | 1.02× |
| period-2 | 3-partition-mll-no | classical net j1 period 2 | 3-partition-mll-no/5 | 51.40 s ✗ | 51.04 s ✗ | 0.99× |
| period-2 | partition-no | classical net j1 period 2 | partition-no/3 | 8.11 s ✗ | 7.82 s ✗ | 0.96× |
| period-2 | partition-no | classical net j1 period 2 | partition-no/4 | > 60 s | > 60 s |  |
| period-2 | partition-yes | classical net j1 period 2 | partition-yes/4 | 6.11 s ✓ | 6.00 s ✓ | 0.98× |
| period-2 | wide-m1 | classical net j1 period 2 | wide-m1/2048 | 856.4 ms ✓ | 860.9 ms ✓ | 1.01× |
| period-2 | wide-m1 | classical net j1 period 2 | wide-m1/256 | 12.2 ms ✓ | 12.4 ms ✓ | 1.02× |
| period-2 | wide-m2 | classical net j1 period 2 | wide-m2/2048 | 840.4 ms ✓ | 855.4 ms ✓ | 1.02× |
| period-2 | wide-m2 | classical net j1 period 2 | wide-m2/256 | 12.4 ms ✓ | 12.4 ms ✓ | 1.01× |
| period-8 | 3-partition-mll-no | classical net j1 period 8 | 3-partition-mll-no/4 | 5.01 s ✗ | 4.96 s ✗ | 0.99× |
| period-8 | 3-partition-mll-no | classical net j1 period 8 | 3-partition-mll-no/5 | > 60 s | > 60 s |  |
| period-8 | partition-no | classical net j1 period 8 | partition-no/3 | > 60 s | > 60 s |  |
| period-8 | partition-no | classical net j1 period 8 | partition-no/4 | > 60 s | > 60 s |  |
| period-8 | partition-yes | classical net j1 period 8 | partition-yes/4 | > 60 s | > 60 s |  |
| period-8 | wide-m1 | classical net j1 period 8 | wide-m1/2048 | 520.8 ms ✓ | 494.5 ms ✓ | 0.95× |
| period-8 | wide-m1 | classical net j1 period 8 | wide-m1/256 | 7.3 ms ✓ | 7.2 ms ✓ | 0.99× |
| period-8 | wide-m2 | classical net j1 period 8 | wide-m2/2048 | 503.9 ms ✓ | 491.0 ms ✓ | 0.97× |
| period-8 | wide-m2 | classical net j1 period 8 | wide-m2/256 | 7.6 ms ✓ | 7.2 ms ✓ | 0.94× |

## The passes under one bias against the default

### lltp-rarer against lltp-intuitionistic

#### Verdicts that differ

None.

#### Decided by lltp-rarer, not by lltp-intuitionistic

| file | family | configuration | problem | lltp-rarer | lltp-intuitionistic |
|---|---|---|---|--:|--:|
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | Diffusion2D_2D8_gradient_40x40_100_5_1.p | 4.50 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | ResAllocation_RAS-C-100_5_1.p | 2.71 s ✓ | > 5 s |

#### By group

Problems in both; decided (late: after the time limit); decided by one and not the other; where the undecided ended (`time` the time limit or `killed` past it, `bound` the copy bound, `depth` the recursion limit, `wide` a context too wide to split, `crash` out of memory); the median ratio of lltp-intuitionistic's time to lltp-rarer's on the problems both decide within the limit (how many); and the problems only lltp-intuitionistic has.

| file | family | configuration | problems | lltp-rarer decided | lltp-intuitionistic decided | only lltp-rarer | only lltp-intuitionistic | lltp-rarer ended | lltp-intuitionistic ended | lltp-intuitionistic/lltp-rarer | new |
|---|---|---|--:|--:|--:|--:|--:|---|---|--:|--:|
|  | ILL/ILLTP-LCL-01 | intuitionistic auto j1 | 2 | 0 | 0 | 0 | 0 | 2 bound | 2 bound | – | 0 |
|  | ILL/ILLTP-LCL-cbn | intuitionistic auto j1 | 2 | 1 | 2 | 0 | 1 | 1 bound | – | 0.92× (1) | 0 |
|  | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 | 2 | 0 | 0 | 0 | 0 | 2 bound | 2 bound | – | 0 |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 252 | 11 | 11 | 0 | 0 | 7 time, 209 bound, 22 depth, 3 killed | 7 time, 209 bound, 22 depth, 3 killed | 1.05× (11) | 0 |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 252 | 34 | 35 | 0 | 1 | 8 time, 186 bound, 22 depth, 2 killed | 8 time, 185 bound, 22 depth, 2 killed | 2.35× (34) | 0 |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 252 | 39 | 39 | 0 | 0 | 27 time, 173 bound, 10 depth, 3 killed | 27 time, 173 bound, 10 depth, 3 killed | 1.03× (39) | 0 |
|  | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | 19 | 11 | 11 | 0 | 0 | 8 bound | 8 bound | 1.00× (11) | 0 |
|  | ILL/ILLTP-SYN-cbn | intuitionistic auto j1 | 19 | 16 | 16 | 0 | 0 | 3 bound | 3 bound | 1.09× (16) | 0 |
|  | ILL/ILLTP-SYN-cbv | intuitionistic auto j1 | 19 | 16 | 16 | 0 | 0 | 3 bound | 3 bound | 1.00× (16) | 0 |
|  | ILL/KLE-01 | intuitionistic auto j1 | 88 | 50 | 50 | 0 | 0 | 38 bound | 38 bound | 1.00× (50) | 0 |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | 222 | 162 | 162 | 0 | 0 | 60 bound | 60 bound | 1.00× (162) | 0 |
|  | ILL/KLE-IMP-CONJ/ALT | intuitionistic auto j1 | 27 | 27 | 27 | 0 | 0 | – | – | 1.05× (27) | 0 |
|  | ILL/KLE-IMP-CONJ/NON-THEOREMS | intuitionistic auto j1 | 22 | 22 | 22 | 0 | 0 | – | – | 0.90× (22) | 0 |
|  | ILL/KLE-cbn | intuitionistic auto j1 | 88 | 68 | 68 | 0 | 0 | 20 bound | 20 bound | 1.07× (68) | 0 |
|  | ILL/KLE-cbv | intuitionistic auto j1 | 88 | 67 | 67 | 0 | 0 | 21 bound | 21 bound | 1.00× (67) | 0 |
|  | ILL/Non-theorems | intuitionistic auto j1 | 1 | 0 | 0 | 0 | 0 | 1 bound | 1 bound | – | 0 |
|  | ILL/misc | intuitionistic auto j1 | 3 | 3 | 3 | 0 | 0 | – | – | 1.07× (3) | 0 |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | 3137 | 442 | 1520 (2 late) | 2 | 1080 | 2347 time, 313 bound, 32 depth, 3 crash | 1594 time, 2 bound, 7 killed, 14 crash | 1.29× (440) | 0 |

### lltp-forward against lltp-intuitionistic

#### Verdicts that differ

None.

#### Decided by lltp-forward, not by lltp-intuitionistic

| file | family | configuration | problem | lltp-forward | lltp-intuitionistic |
|---|---|---|---|--:|--:|
|  | ILL/ILLTP-LCL-01 | intuitionistic auto j1 | LCL230+1.p | 51 µs ✗ | 43 µs ? bound |
|  | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 | LCL181+1.p | 51 µs ✗ | 39 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ106+1.p | 164 µs ✓ | 66 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ107+1.002.p | 730 µs ✓ | 86 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ107+1.003.p | 9.1 ms ✓ | 120 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ107+1.004.p | 97.0 ms ✓ | 169 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ201+1.001.p | 2.0 ms ✓ | 106 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ201+1.002.p | 1.02 s ✓ | 284 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ202+1.002.p | 694 µs ✓ | 354 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ202+1.003.p | 182.7 ms ✓ | 2.2 ms ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.002.p | 71 µs ✓ | 65 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.003.p | 210 µs ✓ | 99 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.004.p | 878 µs ✓ | 154 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.005.p | 4.8 ms ✓ | 200 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.006.p | 31.7 ms ✓ | 300 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.007.p | 217.4 ms ✓ | 398 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.008.p | 1.49 s ✓ | 537 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.002.p | 49 µs ✓ | 28 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.003.p | 67 µs ✓ | 32 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.004.p | 380 µs ✓ | 35 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.005.p | 589 µs ✓ | 41 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.006.p | 877 µs ✓ | 44 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.007.p | 1.7 ms ✓ | 49 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.008.p | 53.3 ms ✓ | 63 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.009.p | 97.2 ms ✓ | 65 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.010.p | 163.3 ms ✓ | 71 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.011.p | 295.2 ms ✓ | 76 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.012.p | 519.3 ms ✓ | 84 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.013.p | 1.02 s ✓ | 102 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.014.p | 1.75 s ✓ | 104 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.015.p | 2.93 s ✓ | 111 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.001.p | 230 µs ✓ | 39 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.002.p | 1.2 ms ✓ | 37 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.003.p | 6.8 ms ✓ | 36 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.004.p | 33.7 ms ✓ | 39 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.005.p | 149.3 ms ✓ | 39 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.006.p | 726.3 ms ✓ | 43 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.007.p | 3.30 s ✓ | 42 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ206+1.003.p | 719 µs ✓ | 118 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ206+1.004.p | 22.7 ms ✓ | 215 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ206+1.005.p | 1.31 s ✓ | 392 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ209+1.002.p | 62 µs ✗ | 45 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ212+1.001.p | 115 µs ✓ MISMATCH | 30 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ212+1.002.p | 420 µs ✗ | 65 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ106+1.p | 45 µs ✓ | 101 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ107+1.002.p | 335 µs ✓ | 71 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ107+1.003.p | 2.9 ms ✓ | 99 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ107+1.004.p | 30.0 ms ✓ | 133 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ201+1.001.p | 160 µs ✓ | 46 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ201+1.002.p | 1.7 ms ✓ | 105 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ201+1.003.p | 8.6 ms ✓ | 204 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ201+1.004.p | 441.0 ms ✓ | 392 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ201+1.005.p | 1.91 s ✓ | 645 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ202+1.002.p | 483 µs ✓ | 244 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ202+1.003.p | 116.4 ms ✓ | 1.5 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.002.p | 66 µs ✓ | 51 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.003.p | 145 µs ✓ | 79 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.004.p | 544 µs ✓ | 107 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.005.p | 2.7 ms ✓ | 153 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.006.p | 16.6 ms ✓ | 213 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.007.p | 110.2 ms ✓ | 271 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.008.p | 740.7 ms ✓ | 385 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.003.p | 25 µs ✓ | 78 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.004.p | 33 µs ✓ | 80 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.005.p | 40 µs ✓ | 85 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.006.p | 48 µs ✓ | 92 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.007.p | 62 µs ✓ | 100 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.008.p | 74 µs ✓ | 110 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.009.p | 96 µs ✓ | 115 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.010.p | 116 µs ✓ | 128 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.011.p | 137 µs ✓ | 132 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.012.p | 166 µs ✓ | 148 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.013.p | 202 µs ✓ | 155 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.014.p | 226 µs ✓ | 168 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.015.p | 269 µs ✓ | 185 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.016.p | 344 µs ✓ | 208 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.017.p | 429 µs ✓ | 210 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.018.p | 448 µs ✓ | 227 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.019.p | 507 µs ✓ | 245 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.020.p | 590 µs ✓ | 265 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.001.p | 44 µs ✓ | 105 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.002.p | 112 µs ✓ | 129 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.003.p | 457 µs ✓ | 169 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.004.p | 2.7 ms ✓ | 221 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.005.p | 19.4 ms ✓ | 278 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.006.p | 149.4 ms ✓ | 370 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.007.p | 1.28 s ✓ | 505 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ206+1.004.p | 268 µs ✓ | 237 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ206+1.005.p | 2.4 ms ✓ | 453 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ206+1.006.p | 24.2 ms ✓ | 927 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ206+1.007.p | 403.8 ms ✓ | 1.8 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ207+1.001.p | 124 µs ✗ | 46 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ209+1.002.p | 51 µs ✗ | 104 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.001.p | 25 µs ✗ | 82 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.003.p | 54 µs ✗ | 128 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.004.p | 115 µs ✗ | 246 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.005.p | 234 µs ✗ | 398 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.006.p | 545 µs ✗ | 809 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.007.p | 1.2 ms ✗ | 1.6 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.008.p | 2.9 ms ✗ | 3.5 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.009.p | 8.1 ms ✗ | 9.0 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.010.p | 22.3 ms ✗ | 22.4 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.011.p | 65.3 ms ✗ | 57.1 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.012.p | 173.3 ms ✗ | 153.1 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.013.p | 518.4 ms ✗ | 411.3 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.014.p | 1.33 s ✗ | 1.10 s ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.015.p | 3.50 s ✗ | 2.87 s ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ103+1.p | 81 µs ✗ MISMATCH | 42 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ105+1.003.p | 47 µs ✗ MISMATCH | 36 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ105+1.004.p | 59 µs ✗ MISMATCH | 37 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ106+1.p | 132 µs ✓ | 70 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ107+1.004.p | 1.6 ms ✓ | 165 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ201+1.001.p | 148 µs ✓ | 98 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ201+1.002.p | 1.3 ms ✓ | 238 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ201+1.003.p | 5.7 ms ✓ | 502 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ201+1.004.p | 61.7 ms ✓ | 957 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ201+1.005.p | 221.1 ms ✓ | 1.8 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ201+1.006.p | 615.4 ms ✓ | 2.8 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ201+1.007.p | 1.47 s ✓ | 4.5 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.002.p | 64 µs ✓ | 59 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.003.p | 192 µs ✓ | 83 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.004.p | 810 µs ✓ | 126 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.005.p | 4.9 ms ✓ | 166 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.006.p | 32.3 ms ✓ | 236 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.007.p | 222.0 ms ✓ | 347 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.008.p | 1.70 s ✓ | 425 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.002.p | 64 µs ✓ | 37 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.003.p | 82 µs ✓ | 41 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.004.p | 433 µs ✓ | 45 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.005.p | 636 µs ✓ | 48 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.006.p | 955 µs ✓ | 55 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.007.p | 1.8 ms ✓ | 64 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.008.p | 56.9 ms ✓ | 71 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.009.p | 97.4 ms ✓ | 77 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.010.p | 170.6 ms ✓ | 81 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.011.p | 308.4 ms ✓ | 93 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.012.p | 555.6 ms ✓ | 98 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.013.p | 1.03 s ✓ | 113 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.014.p | 1.78 s ✓ | 125 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.015.p | 2.92 s ✓ | 134 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.002.p | 211 µs ✓ | 85 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.003.p | 482 µs ✓ | 126 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.004.p | 2.8 ms ✓ | 193 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.005.p | 5.6 ms ✓ | 257 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.006.p | 10.5 ms ✓ | 358 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.007.p | 18.6 ms ✓ | 455 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.008.p | 121.4 ms ✓ | 618 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.009.p | 209.9 ms ✓ | 755 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.010.p | 343.1 ms ✓ | 1.1 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.011.p | 559.2 ms ✓ | 1.3 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.012.p | 903.0 ms ✓ | 1.6 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.013.p | 1.35 s ✓ | 2.0 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.014.p | 2.04 s ✓ | 2.6 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.015.p | 4.12 s ✓ | 2.9 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ206+1.004.p | 1.3 ms ✓ | 309 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ206+1.005.p | 18.0 ms ✓ | 655 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ206+1.006.p | 402.9 ms ✓ | 1.5 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ207+1.001.p | 487 µs ✗ | 71 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ209+1.002.p | 67 µs ✗ | 43 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ210+1.001.p | 46 µs ✗ | 31 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ212+1.001.p | 50 µs ✓ MISMATCH | 26 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ212+1.002.p | 303 µs ✗ | 60 µs ? bound |
|  | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | SYN044+1.p | 107 µs ✓ | 55 µs ? bound |
|  | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | SYN045+1.p | 203 µs ✓ | 97 µs ? bound |
|  | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | SYN388+1.p | 37 µs ✗ | 30 µs ? bound |
|  | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | SYN389+1.p | 31 µs ✗ | 23 µs ? bound |
|  | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | SYN391+1.p | 364 µs ✓ | 86 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE004+1.p | 51 µs ✓ | 39 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE018+1.p | 138 µs ✓ | 93 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE026+1.p | 329 µs ✓ | 97 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE027+1.p | 475 µs ✓ | 88 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE028+1.p | 162 µs ✓ | 63 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE029+1.p | 374 µs ✓ | 77 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE030+1.p | 2.2 ms ✓ | 58 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE031+1.p | 14.2 ms ✓ | 70 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE032+1.p | 92 µs ✓ | 73 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE033+1.p | 108 µs ✓ | 76 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE034+1.p | 122 µs ✓ | 83 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE035+1.p | 132 µs ✓ | 97 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE036+1.p | 170 µs ✓ | 75 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE037+1.p | 175 µs ✓ | 91 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE038+1.p | 929 µs ✓ | 57 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE043+1.p | 150 µs ✓ | 81 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE044+1.p | 270 µs ✓ | 102 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE058+1.p | 138 µs ✓ | 31 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE061+1.p | 32 µs ✓ | 24 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE062+1.p | 76 µs ✓ | 32 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE066+1.p | 50 µs ✓ | 30 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE067+1.p | 99 µs ✓ | 66 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE068+1.p | 47 µs ✗ | 43 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE069+1.p | 475 µs ✗ MISMATCH | 46 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE073+1.p | 64 µs ✓ | 40 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE076+1.p | 151 µs ✓ | 51 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE080+1.p | 62 µs ✓ | 47 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE081+1.p | 62 µs ✓ | 47 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE082+1.p | 112 µs ✓ | 43 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE084+1.p | 9.5 ms ✓ | 128 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_14_01.p | 72 µs ✓ | 58 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_14_CBV.p | 55 µs ✓ | 46 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_22_01.p | 294 µs ✓ | 101 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_23_01.p | 464 µs ✓ | 85 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_23_CBN.p | 57 µs ✓ | 109 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_23_CBV.p | 229 µs ✓ | 58 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_24_01.p | 175 µs ✓ | 67 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_24_CBN.p | 44 µs ✓ | 95 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_24_CBV.p | 163 µs ✓ | 46 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_25_01.p | 368 µs ✓ | 80 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_25_CBN.p | 96 µs ✓ | 104 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_25_CBV.p | 168 µs ✓ | 60 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_26_01.p | 256 µs ✓ | 52 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_26_CBN.p | 58 µs ✓ | 51 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_26_CBV.p | 75 µs ✓ | 62 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_27_01.p | 736 µs ✓ | 68 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_27_CBN.p | 89 µs ✓ | 100 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_27_CBV.p | 179 µs ✓ | 50 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_28_01.p | 85 µs ✓ | 72 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_29_01.p | 108 µs ✓ | 76 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_30_01.p | 126 µs ✓ | 85 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_31_01.p | 134 µs ✓ | 93 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_32_01.p | 111 µs ✓ | 70 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_44_01.p | 34 µs ✓ | 26 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_45_01.p | 78 µs ✓ | 35 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_45_CBN.p | 26 µs ✓ | 78 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_45_CBV.p | 38 µs ✓ | 27 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_46_01.p | 52 µs ✓ | 32 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_46_CBN.p | 23 µs ✓ | 19 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_46_CBV.p | 34 µs ✓ | 29 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_47_01.p | 105 µs ✓ | 69 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_48_01.p | 61 µs ✓ | 41 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_49_01.p | 62 µs ✓ | 49 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_4_01.p | 48 µs ✓ | 39 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_50_01.p | 117 µs ✓ | 80 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_51_01.p | 217 µs ✓ | 99 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_51_CBN.p | 68 µs ✓ | 126 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_51_CBV.p | 102 µs ✓ | 69 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_52_01.p | 171 µs ✓ | 84 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_52_CBN.p | 51 µs ✓ | 125 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_52_CBV.p | 85 µs ✓ | 78 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_53_01.p | 93 µs ✓ | 52 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_54_01.p | 65 µs ✓ | 51 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_55_01.p | 71 µs ✓ | 45 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_56_01.p | 142 µs ✓ | 51 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_56_CBV.p | 57 µs ✓ | 41 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_57_01.p | 163 µs ✓ | 55 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_57_CBN.p | 39 µs ✓ | 91 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_57_CBV.p | 57 µs ✓ | 43 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_58_01.p | 151 µs ✓ | 52 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_58_CBN.p | 43 µs ✓ | 81 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_58_CBV.p | 61 µs ✓ | 49 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_59_01.p | 177 µs ✓ | 55 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_59_CBN.p | 46 µs ✓ | 102 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_59_CBV.p | 71 µs ✓ | 41 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_60_01.p | 156 µs ✓ | 83 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_60_CBV.p | 71 µs ✓ | 61 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_61_01.p | 170 µs ✓ | 61 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_61_CBN.p | 52 µs ✓ | 33 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_61_CBV.p | 91 µs ✓ | 49 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE017+1.p | 33 µs ✗ MISMATCH | 25 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE018+1.p | 43 µs ✓ | 33 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE027+1.p | 59 µs ✓ | 107 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE028+1.p | 37 µs ✓ | 83 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE029+1.p | 99 µs ✓ | 113 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE030+1.p | 227 µs ✓ | 32 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE031+1.p | 229 µs ✓ | 85 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE038+1.p | 43 µs ✓ | 97 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE058+1.p | 33 µs ✓ | 70 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE062+1.p | 25 µs ✓ | 74 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE068+1.p | 39 µs ✗ | 35 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE069+1.p | 85 µs ✗ MISMATCH | 26 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE078+1.p | 51 µs ✗ MISMATCH | 91 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE084+1.p | 722 µs ✓ | 128 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE086+1.p | 153 µs ✗ MISMATCH | 90 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE088+1.p | 172 µs ✗ MISMATCH | 40 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE017+1.p | 120 µs ✗ MISMATCH | 36 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE018+1.p | 84 µs ✓ | 71 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE027+1.p | 222 µs ✓ | 58 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE028+1.p | 80 µs ✓ | 45 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE029+1.p | 167 µs ✓ | 60 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE030+1.p | 276 µs ✓ | 53 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE031+1.p | 1.6 ms ✓ | 49 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE038+1.p | 149 µs ✓ | 41 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE058+1.p | 54 µs ✓ | 25 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE062+1.p | 37 µs ✓ | 27 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE069+1.p | 44 µs ✗ MISMATCH | 37 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE076+1.p | 58 µs ✓ | 37 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE078+1.p | 48 µs ✗ MISMATCH | 31 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE082+1.p | 51 µs ✓ | 38 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE084+1.p | 2.6 ms ✓ | 107 µs ? bound |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | BridgeAndVehicles-V50-P20-N50-unfolded_10_1.p | 4.75 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | BridgeAndVehicles-V80-P20-N20-unfolded_10_1.p | 3.73 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | BridgeAndVehicles-V80-P50-N20-unfolded_10_1.p | 4.13 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CSRepetitions_cs_repetitions-3-unfolded_10_1.p | 2.98 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CircularTrains_CircularTrain-024_100_1.p | 2.03 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CircularTrains_CircularTrain-024_50_1.p | 1.62 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudDeployment_deploy_5_b_5_1.p | 4.08 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_01_10_1.p | 2.27 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_02_10_1.p | 2.25 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_03_10_1.p | 2.25 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_04_10_1.p | 2.25 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_05_10_1.p | 2.31 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_06_10_1.p | 2.24 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_07_10_1.p | 2.24 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_08_10_1.p | 2.24 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_09_10_1.p | 2.26 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_10_10_1.p | 2.27 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_11_10_1.p | 2.26 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_12_10_1.p | 2.25 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_13_10_1.p | 2.26 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_14_10_1.p | 2.25 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_15_10_1.p | 2.26 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_16_10_1.p | 2.28 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_17_10_1.p | 2.26 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_18_10_1.p | 2.27 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_19_10_1.p | 2.26 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | CloudReconfiguration_reconf_3_20_10_1.p | 2.26 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | DES_des_05_a_20_1.p | 1.97 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | DES_des_10_a_20_1.p | 2.52 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | DES_des_20_a_20_1.p | 3.55 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | DES_des_30_a_20_1.p | 4.53 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | DLCround_dlcro_04_b_5_1.p | 2.39 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | DLCround_dlcro_05_b_5_1.p | 4.08 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | DLCround_dlcro_11_a_5_1.p | 1.98 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | DLCround_dlcro_13_a_5_1.p | 4.12 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | DLCshifumi_dlcsh_2_b_5_1.p | 4.11 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | DNAwalker_dnawalk-08_20_1.p | 4.30 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | DNAwalker_dnawalk-09_20_1.p | 4.63 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | Diffusion2D_2D8_gradient_40x40_100_5_1.p | 3.90 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | ERK_erk-000100_20_1.p | 1.75 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | Eratosthenes_eratosthenes-020_10_1.p | 2.69 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | Eratosthenes_eratosthenes-020_20_1.p | 3.66 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | FMS-10_20_1.p | 4.15 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | FMS-5_20_1.p | 2.38 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | FlexibleBarrier_flexbar_08_a_10_1.p | 2.39 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | FlexibleBarrier_flexbar_14_b_5_1.p | 4.17 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | HexagonalGrid_hxg_226_5_1.p | 2.68 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | HypertorusGrid_ht_d2k1p8b00_50_1.p | 3.88 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | JoinFreeModules_joinFree-5_10_1.p | 3.74 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | Kanban-100_20_1.p | 2.35 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | Kanban-200_20_1.p | 4.41 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | Kanban-50_20_1.p | 1.33 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | LamportFastMutEx_lamport_fmea-3_100_1.p | 3.28 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | LamportFastMutEx_lamport_fmea-8_10_1.p | 4.68 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | Peterson-3_20_1.p | 4.95 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | PhaseVariation_5-10_phaseVariation_5_1.p | 2.61 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | PolyORBLF_PolyORB-LF-S06-J04-T08-unfolded_5_1.p | 4.16 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | PolyORBLF_PolyORB-LF-S06-J06-T08-unfolded_5_1.p | 4.42 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | QuasiCertifProtocol_QCertifProtocol_06-unfold_10_1.p | 4.75 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | Raft_raft_06_5_1.p | 1.86 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | ResAllocation_RAS-C-50_5_1.p | 2.54 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | Solitaire_soli1_10_1.p | 4.97 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | TokenRing-10-unfolded_10_1.p | 3.41 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | TokenRing-20-unfolded_5_1.p | 3.96 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | TriangularGrid_trg_1-50-0_20_1.p | 1.62 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | TriangularGrid_trg_2-01-1_10_1.p | 2.98 s ✓ | > 5 s |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | TriangularGrid_trg_4-02-2_5_1.p | 3.00 s ✓ | > 5 s |

#### By group

Problems in both; decided (late: after the time limit); decided by one and not the other; where the undecided ended (`time` the time limit or `killed` past it, `bound` the copy bound, `depth` the recursion limit, `wide` a context too wide to split, `crash` out of memory); the median ratio of lltp-intuitionistic's time to lltp-forward's on the problems both decide within the limit (how many); and the problems only lltp-intuitionistic has.

| file | family | configuration | problems | lltp-forward decided | lltp-intuitionistic decided | only lltp-forward | only lltp-intuitionistic | lltp-forward ended | lltp-intuitionistic ended | lltp-intuitionistic/lltp-forward | new |
|---|---|---|--:|--:|--:|--:|--:|---|---|--:|--:|
|  | ILL/ILLTP-LCL-01 | intuitionistic auto j1 | 2 | 1 | 0 | 1 | 0 | 1 bound | 2 bound | – | 0 |
|  | ILL/ILLTP-LCL-cbn | intuitionistic auto j1 | 2 | 2 | 2 | 0 | 0 | – | – | 2.82× (2) | 0 |
|  | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 | 2 | 1 | 0 | 1 | 0 | 1 bound | 2 bound | – | 0 |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 252 | 53 | 11 | 42 | 0 | 135 time, 39 bound, 22 depth, 3 killed | 7 time, 209 bound, 22 depth, 3 killed | 0.96× (11) | 0 |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 252 | 98 | 35 | 63 | 0 | 100 time, 30 bound, 22 depth, 2 killed | 8 time, 185 bound, 22 depth, 2 killed | 2.46× (35) | 0 |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 252 | 94 | 39 | 55 | 0 | 126 time, 19 bound, 10 depth, 3 killed | 27 time, 173 bound, 10 depth, 3 killed | 1.03× (39) | 0 |
|  | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | 19 | 16 | 11 | 5 | 0 | 3 bound | 8 bound | 1.00× (11) | 0 |
|  | ILL/ILLTP-SYN-cbn | intuitionistic auto j1 | 19 | 16 | 16 | 0 | 0 | 3 bound | 3 bound | 1.21× (16) | 0 |
|  | ILL/ILLTP-SYN-cbv | intuitionistic auto j1 | 19 | 16 | 16 | 0 | 0 | 3 bound | 3 bound | 1.00× (16) | 0 |
|  | ILL/KLE-01 | intuitionistic auto j1 | 88 | 80 | 50 | 30 | 0 | 1 time, 7 bound | 38 bound | 1.00× (50) | 0 |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | 222 | 222 | 162 | 60 | 0 | – | 60 bound | 1.00× (162) | 0 |
|  | ILL/KLE-IMP-CONJ/ALT | intuitionistic auto j1 | 27 | 27 | 27 | 0 | 0 | – | – | 1.00× (27) | 0 |
|  | ILL/KLE-IMP-CONJ/NON-THEOREMS | intuitionistic auto j1 | 22 | 22 | 22 | 0 | 0 | – | – | 0.87× (22) | 0 |
|  | ILL/KLE-cbn | intuitionistic auto j1 | 88 | 84 | 68 | 16 | 0 | 4 bound | 20 bound | 1.10× (68) | 0 |
|  | ILL/KLE-cbv | intuitionistic auto j1 | 88 | 82 | 67 | 15 | 0 | 6 bound | 21 bound | 1.00× (67) | 0 |
|  | ILL/Non-theorems | intuitionistic auto j1 | 1 | 0 | 0 | 0 | 0 | 1 bound | 1 bound | – | 0 |
|  | ILL/misc | intuitionistic auto j1 | 3 | 3 | 3 | 0 | 0 | – | – | 1.00× (3) | 0 |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | 3137 | 1576 (2 late) | 1520 (2 late) | 67 | 11 | 1443 time, 93 bound, 11 killed, 14 crash | 1594 time, 2 bound, 7 killed, 14 crash | 1.64× (1507) | 0 |

