# The target set of the focused engine's performance pass

`bench/targets.sh LABEL` runs these 165 sequential runs on two pinned
performance cores (CPUs 2 and 3 of the Intel Core Ultra X9 388H the
baselines are taken on), by day, capped at 8 GiB, into
`bench/targets/LABEL.csv`: 52 on generated families and the problem
file with 300 s each, 113 on a fixed sample of LLTP problems with 5 s
each. The three labels here were taken on 2026-10-02:

- `before`: the engines of the first baseline (the commit that adds the
  target set), whose counters it reproduces on every row both decide;
- `after-search`: after the changes to what the focused engine searches
  (the split search, the canonical choice among equal members, the memo's
  complete failures up to renaming, the atom bias, the forced rule for a
  tensor of literals, the chains without recursion, the loop check's
  hashes);
- `after`: after the changes to what a unit of search costs, which leave
  the counters of every decided row as `after-search` has them.

A fourth label was taken later the same day, and has a section of its
own at the end:

- `after-bias`: the default bias runs a backward and a forward search
  on a sequent with exponentials; without exponentials nothing changed.

Stable sequents and splits are machine-independent on one thread and
come first; a time is the median of up to three runs, the CPU time of
the search where it is 100 ms or more and the wall time below that, since
CPU time comes in ticks of 10 ms. `splits` are submasks enumerated
before and steps of the split search after. An empty cell is a run that
ended at its limit. `plan/reports/15-performance.md` reads the numbers.

## Families and the problem file

| problem | outcome | before: stable sequents / splits | time | after the search changes: stable sequents / splits | time | after the constant factors: time |
|---|---|--:|--:|--:|--:|--:|
| 3-partition-no/4 | refuted | 1 834 321 / 3 870 480 309 | 47 s | 923 / 2 794 | 284 µs | 296 µs |
| 3-partition-no/5 | refuted | 5 434 141 / 23 005 595 829 | 292 s | 971 / 3 294 | 337 µs | 353 µs |
| partition-yes/5 | proved | 31 / 13 972 379 | 170 ms | 30 / 157 | 68 µs | 73 µs |
| partition-yes/6 | proved | 94 / 946 564 520 | 12 s | 93 / 542 | 119 µs | 124 µs |
| partition-yes/7 | time limit → proved |  | > 300 s | 178 / 1 097 | 195 µs | 205 µs |
| partition-no/4 | refuted | 217 / 8 192 777 | 100 ms | 217 / 939 | 125 µs | 129 µs |
| partition-no/5 | time limit → refuted |  | > 300 s | 811 / 4 161 | 454 µs | 449 µs |
| mix/8 | refuted | 14 316 140 / 91 023 866 | 1.81 s | 14 316 140 / 51 741 261 | 2.11 s | 1.58 s |
| mix/9 | refuted | 129 009 092 / 904 636 500 | 19 s | 129 009 092 / 465 310 909 | 20 s | 14 s |
| mix/10 | refuted | 1 161 737 180 / 8 913 554 830 | 206 s | 1 161 737 180 / 4 186 028 717 | 211 s | 148 s |
| mix/11 | time limit |  | > 300 s |  | > 300 s | > 300 s |
| qbf/16#0 | proved | 25 282 / 622 825 073 | 7.19 s | 14 615 / 22 146 | 8.9 ms | 9.5 ms |
| qbf/16#1 | refuted | 3 762 / 114 202 169 | 1.33 s | 2 502 / 4 090 | 1.4 ms | 1.5 ms |
| qbf/16#2 | refuted | 6 346 / 161 265 602 | 1.88 s | 3 975 / 6 214 | 2.2 ms | 2.4 ms |
| qbf/16#3 | proved | 21 840 / 516 826 066 | 6.05 s | 12 420 / 19 728 | 7.2 ms | 7.6 ms |
| qbf/20#0 | refuted | 17 047 / 6 917 772 063 | 80 s | 11 161 / 17 161 | 6.7 ms | 7.3 ms |
| qbf/20#1 | refuted | 29 366 / 9 749 414 876 | 113 s | 17 781 / 26 685 | 11 ms | 12 ms |
| qbf/20#2 | time limit → proved |  | > 300 s | 60 883 / 97 394 | 44 ms | 49 ms |
| qbf/20#3 | refuted | 16 202 / 7 100 259 975 | 83 s | 10 639 / 16 467 | 6.4 ms | 7.0 ms |
| counter/8 | proved | 1 764 046 / 1 835 764 | 120 ms | 14 228 / 42 105 | 1.6 ms | 1.6 ms |
| counter/8 (two-sided) | proved | 218 945 / 234 822 | 15 ms | 701 / 2 697 | 141 µs | 131 µs |
| counter/16 | time limit → proved |  | > 300 s | 473 232 / 2 004 517 | 62 ms | 56 ms |
| counter/16 (two-sided) | time limit → proved |  | > 300 s | 9 141 / 53 727 | 1.4 ms | 1.2 ms |
| counter-over/8 | copy bound | 6 899 911 / 6 996 462 | 490 ms | 19 423 / 59 335 | 2.3 ms | 2.2 ms |
| counter-over/8 (two-sided) | copy bound | 255 521 / 270 098 | 18 ms | 709 / 2 724 | 146 µs | 140 µs |
| growing/16 | copy bound | 409 / 256 | 136 µs | 409 / 256 | 132 µs | 128 µs |
| growing/64 | copy bound | 6 241 / 4 096 | 2.0 ms | 6 241 / 4 096 | 1.6 ms | 1.6 ms |
| growing/256 | copy bound | 98 689 / 65 536 | 79 ms | 98 689 / 65 536 | 42 ms | 44 ms |
| growing/1024 | recursion limit | 392 961 / 261 632 | 560 ms | 392 961 / 261 633 | 260 ms | 270 ms |
| chain/16 | proved | 253 / 1 817 | 244 µs | 253 / 1 817 | 202 µs | 154 µs |
| chain/64 | proved | 4 069 / 127 073 | 22 ms | 4 069 / 127 073 | 12 ms | 5.5 ms |
| chain/128 | proved | 16 325 / 1 032 385 | 280 ms | 16 325 / 1 032 385 | 120 ms | 43 ms |
| chain/256 | proved | 65 413 / 8 323 457 | 2.88 s | 65 413 / 8 323 457 | 600 ms | 340 ms |
| wide-m3/24 | proved | 73 / 11 184 869 | 160 ms | 25 / 454 | 75 µs | 87 µs |
| wide-m3/30 | proved | 91 / 715 827 956 | 11 s | 31 / 688 | 102 µs | 112 µs |
| wide-m4/24 | proved | 73 / 11 184 869 | 160 ms | 25 / 430 | 77 µs | 88 µs |
| wide-m4/28 | proved | 85 / 178 957 039 | 2.66 s | 29 / 572 | 90 µs | 101 µs |
| partition-table/1-1 | proved | 15 / 0 | 35 µs | 15 / 0 | 32 µs | 33 µs |
| partition-table/1-3 | refuted | 13 / 5 161 | 119 µs | 13 / 59 | 28 µs | 28 µs |
| partition-table/2-1-1 | proved | 16 / 7 291 | 148 µs | 15 / 57 | 39 µs | 42 µs |
| partition-table/1-1-4 | refuted | 55 / 147 635 | 2.0 ms | 55 / 233 | 50 µs | 51 µs |
| partition-table/2-2-1-1 | proved | 29 / 148 482 | 1.9 ms | 28 / 124 | 57 µs | 58 µs |
| partition-table/1-2-5 | refuted | 55 / 417 989 | 5.7 ms | 55 / 251 | 55 µs | 56 µs |
| partition-table/1-1-2-4 | proved | 12 / 35 545 | 480 µs | 11 / 39 | 43 µs | 46 µs |
| partition-table/1-1-1-5 | refuted | 217 / 4 457 149 | 59 ms | 217 / 863 | 121 µs | 127 µs |
| partition-table/2-3-2-1 | proved | 29 / 445 994 | 5.7 ms | 28 / 132 | 54 µs | 61 µs |
| partition-table/3-3-3-1 | refuted | 217 / 8 192 777 | 100 ms | 217 / 939 | 138 µs | 135 µs |
| partition-table/1-2-3-4-5-5 | proved | 22 / 1 790 225 865 | 23 s | 21 / 127 | 73 µs | 78 µs |
| partition-table/1-1-1-1-1-7 | refuted | 2 917 / 4 362 085 393 | 56 s | 2 917 / 10 691 | 1.2 ms | 1.2 ms |
| partition-table/2-2-2-2-2-2-9-1 | time limit → proved |  | > 300 s | 45 / 298 | 108 µs | 111 µs |
| chain-over/12 | copy bound | 70 / 244 | 63 µs | 70 / 260 | 52 µs | 50 µs |
| cancellation/3-partition-4 | proved | 1 834 323 / 3 870 480 309 | 48 s | 925 / 2 794 | 283 µs | 301 µs |

## The LLTP sample

By how the first baseline's intuitionistic pass (5 s, a copy bound of 3,
a recursion limit of 2 048) ended each problem; the last two groups ran
with 25 s and 55 s before the kill, the last under a recursion limit of
16 384.

| the first baseline's problems that | runs | before | after-search |
|---|--:|---|---|
| ended at the time limit | 24 | 24 time limit | 24 time limit |
| ended at the copy bound | 24 | 24 copy bound | 24 copy bound |
| ended at the recursion limit | 24 | 24 recursion limit | 4 proved, 3 copy bound, 2 recursion limit, 15 time limit |
| had a context too wide | 24 | 24 too wide | 3 proved, 21 time limit |
| missed their stop | 13 | 4 time limit, 9 killed | 4 proved, 9 time limit |
| under a recursion limit of 16 384 | 4 | 1 proved, 3 killed | 3 proved, 1 time limit |
| all | 113 | 1 proved, 24 copy bound, 24 recursion limit, 28 time limit, 24 too wide, 12 killed | 14 proved, 27 copy bound, 2 recursion limit, 70 time limit |

| the first baseline's problems that | runs | before | after |
|---|--:|---|---|
| ended at the time limit | 24 | 24 time limit | 1 copy bound, 23 time limit |
| ended at the copy bound | 24 | 24 copy bound | 24 copy bound |
| ended at the recursion limit | 24 | 24 recursion limit | 4 proved, 3 copy bound, 2 recursion limit, 15 time limit |
| had a context too wide | 24 | 24 too wide | 3 proved, 21 time limit |
| missed their stop | 13 | 4 time limit, 9 killed | 4 proved, 9 time limit |
| under a recursion limit of 16 384 | 4 | 1 proved, 3 killed | 3 proved, 1 time limit |
| all | 113 | 1 proved, 24 copy bound, 24 recursion limit, 28 time limit, 24 too wide, 12 killed | 14 proved, 28 copy bound, 2 recursion limit, 69 time limit |

No run of `after` is killed, and the latest stops 5.27 s after its start
under its 5 s limit; before, 12 were killed and 5 more stopped 10 s to
26 s after theirs.

## The default bias

`after-bias` against `after`, the same 165 runs. No row that `after`
decides is lost, no verdict differs, every proof is checked. The 40
decided rows on sequents without exponentials have the stable sequents,
splits, memo hits and memo entries of `after` (their times are 0.95 to
1.14 of it, 1.00 in the median); the rows below are those with
exponentials. Under `after-bias` their counters are the two searches'
together, each as far as it ran when the other decided.

| problem | `after` | stable sequents / splits | time | `after-bias` | stable sequents / splits | time |
|---|---|--:|--:|---|--:|--:|
| counter/8 | proved | 14 228 / 42 105 | 1.6 ms | proved | 47 / 151 | 84 µs |
| counter/8 (two-sided) | proved | 701 / 2 697 | 131 µs | proved | 47 / 151 | 89 µs |
| counter/16 | proved | 473 232 / 2 004 517 | 56 ms | proved | 404 / 1 529 | 244 µs |
| counter/16 (two-sided) | proved | 9 141 / 53 727 | 1.2 ms | proved | 404 / 1 529 | 250 µs |
| counter-over/8 | copy bound | 19 423 / 59 335 | 2.2 ms | refuted | 64 / 245 | 91 µs |
| counter-over/8 (two-sided) | copy bound | 709 / 2 724 | 140 µs | refuted | 64 / 245 | 92 µs |
| growing/16 | copy bound | 409 / 256 | 128 µs | copy bound | 1 396 / 900 | 340 µs |
| growing/64 | copy bound | 6 241 / 4 096 | 1.6 ms | copy bound | 6 241 / 4 096 | 1.6 ms |
| growing/256 | copy bound | 98 689 / 65 536 | 44 ms | copy bound | 98 689 / 65 536 | 43 ms |
| growing/1024 | recursion limit | 392 961 / 261 633 | 270 ms | recursion limit | 392 961 / 261 633 | 260 ms |
| chain/16 | proved | 253 / 1 817 | 154 µs | proved | 253 / 1 817 | 155 µs |
| chain/64 | proved | 4 069 / 127 073 | 5.5 ms | proved | 4 069 / 127 073 | 5.4 ms |
| chain/128 | proved | 16 325 / 1 032 385 | 43 ms | proved | 16 325 / 1 032 385 | 42 ms |
| chain/256 | proved | 65 413 / 8 323 457 | 340 ms | proved | 65 413 / 8 323 457 | 340 ms |
| chain-over/12 | copy bound | 70 / 260 | 50 µs | copy bound | 96 / 414 | 112 µs |

The counter's clauses are Horn, so the forward search runs within its
own bound of 30 copies and proves in a chain of 7 and of 15 steps what
the backward search proves within 3 and 4 copies a branch; the counter
with the unreachable goal is refuted, the forward search having run out
of markings. On `growing` and `chain` the two rules give every atom the
same literal, so one search runs (`growing` at 16 within the forward
bound of 30).

The LLTP sample:

| the first baseline's problems that | runs | `after` | `after-bias` |
|---|--:|---|---|
| ended at the time limit | 24 | 1 copy bound, 23 time limit | 16 proved, 8 time limit |
| ended at the copy bound | 24 | 24 copy bound | 2 proved, 20 copy bound, 2 time limit |
| ended at the recursion limit | 24 | 4 proved, 3 copy bound, 2 recursion limit, 15 time limit | 12 proved, 2 recursion limit, 10 time limit |
| had a context too wide | 24 | 3 proved, 21 time limit | 12 proved, 12 time limit |
| missed their stop | 13 | 4 proved, 9 time limit | 8 proved, 5 time limit |
| under a recursion limit of 16 384 | 4 | 3 proved, 1 time limit | 3 proved, 1 time limit |
| all | 113 | 14 proved, 28 copy bound, 2 recursion limit, 69 time limit | 53 proved, 20 copy bound, 2 recursion limit, 38 time limit |

The latest stop of `after-bias` is 5.26 s after the start. On the 109
sampled problems that run under the default limits, against one run
under each explicit bias with the same engine (5 s, one pinned core;
those three runs are not committed):

| | proved | refuted | copy bound or recursion limit | time limit |
|---|--:|--:|--:|--:|
| `--bias rarer` | 11 | 0 | 29 | 69 |
| `--bias factors` | 23 | 0 | 78 | 8 |
| `--bias factors --copies 10` | 48 | 1 | 23 | 37 |
| the default, `after-bias` | 50 | 0 | 22 | 37 |

The default decides all 26 problems that either explicit bias decides at
the default bound. Against `--bias factors --copies 10` it has every
Petri net of the 49 and six more (nets of 20 and 50 steps, within the
forward bound of 30), and lacks five translations of intuitionistic
problems that a copy bound of 10 decides under either bias: they are
not Horn, so the forward search keeps to `--copies` there. The 44
problems that the default and an explicit run both decide take the
default 6.5 s together where the better explicit run takes 4.1 s; the
ratio is 1.6 in the median and 4.4 at most (64 ms against 14 ms).
