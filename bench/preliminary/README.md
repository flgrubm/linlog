# Preliminary results

The rows of the benchmark run of 2026-09-30, 08:12 to 09:34, on an Intel
Core Ultra X9 388H, stopped before its end. They are not a baseline: the
machine was shared with other work, the four sequential streams were
pinned to its performance cores, and the build predates the `cpu_ms` and
`wait_ms` columns, so a time here is an upper bound with some noise. The
verdicts and the engine's counters (`nodes`, `memo_hits`, `memo_entries`,
`splits`, `links`, `tests`) do not depend on the machine: on one thread
the search is deterministic.

Complete: every family on one thread (`families.csv`), focus against net
on the multiplicative families and the problem file (`engines.csv`), the
net engine's test period (`period-*.csv`), the intuitionistic mode of the
families that have one (`intuitionistic.csv`). Cut off: the LLTP library,
at 4 022 of 4 495 problems intuitionistically (`lltp-intuitionistic.csv`)
and 3 792 of 4 512 classically (`lltp-classical.csv`, in reverse order).
Never run: more than one thread, the portfolio, the raised copy bound and
recursion limit.

`bench/baseline.sh` writes the baseline proper to `bench/results/` and
`bench/RESULTS.md` when the machine is free for a night. The tables below
are `linlog-bench summary bench/preliminary/*.csv`.

## Solved within the time limit

Sequential problems that waited for a CPU for over 1 % of their time (another process slowed them down; `†` in the tables below): 0.

| family | configuration | engines | problems | solved | proved | refuted | timeout | bound | other | mismatch | median solved | total solved |
|---|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| partition-yes | engines: classical focus j1 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 182.6 ms | 12.81 s |
| partition-yes | engines: classical net j1 | net | 3 | 1 | 1 | 0 | 2 | 0 | 0 | 0 | 2.76 s | 2.76 s |
| partition-no | engines: classical focus j1 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 113.8 ms | 118.1 ms |
| partition-no | engines: classical net j1 | net | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 3.51 s | 3.51 s |
| 3-partition-mll-yes | engines: classical focus j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 35 µs | 132 µs |
| 3-partition-mll-yes | engines: classical net j1 | net | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 115.1 ms | 5.98 s |
| 3-partition-mll-no | engines: classical focus j1 | focus | 4 | 4 | 0 | 4 | 0 | 0 | 0 | 0 | 17 µs | 67 µs |
| 3-partition-mll-no | engines: classical net j1 | net | 4 | 2 | 0 | 2 | 2 | 0 | 0 | 0 | 57.19 s | 59.03 s |
| wide-m1 | engines: classical focus j1 | focus | 6 | 4 | 4 | 0 | 0 | 0 | 2 | 0 | 178.4 ms | 45.79 s |
| wide-m1 | engines: classical net j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 227 µs | 649.1 ms |
| wide-m2 | engines: classical focus j1 | focus | 6 | 4 | 4 | 0 | 0 | 0 | 2 | 0 | 172.8 ms | 43.88 s |
| wide-m2 | engines: classical net j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 220 µs | 632.8 ms |
| wide-m3 | engines: classical focus j1 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 171.0 ms | 11.07 s |
| wide-m3 | engines: classical net j1 | net | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 163 µs | 454 µs |
| wide-m4 | engines: classical focus j1 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 172.8 ms | 2.93 s |
| wide-m4 | engines: classical net j1 | net | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 207 µs | 542 µs |
| additive | engines: classical focus j1 | focus | 4 | 3 | 3 | 0 | 0 | 0 | 1 | 0 | 336.2 ms | 6.39 s |
| partition-table | engines: classical focus j1 | focus | 13 | 12 | 6 | 6 | 1 | 0 | 0 | 0 | 5.8 ms | 82.22 s |
| partition-table | engines: classical net j1 | net | 13 | 9 | 5 | 4 | 4 | 0 | 0 | 0 | 50.8 ms | 16.78 s |
| cancellation | engines: classical focus j1 | focus | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 51.20 s | 51.20 s |
| 3-partition-yes | families: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 2.1 ms | 43.9 ms |
| 3-partition-no | families: classical auto j1 | focus | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 52.64 s | 52.64 s |
| 3-partition-mll-yes | families: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 31 µs | 124 µs |
| 3-partition-mll-no | families: classical auto j1 | focus | 4 | 4 | 0 | 4 | 0 | 0 | 0 | 0 | 15 µs | 59 µs |
| partition-yes | families: classical auto j1 | focus | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 191.7 ms | 13.13 s |
| partition-no | families: classical auto j1 | focus | 3 | 2 | 0 | 2 | 1 | 0 | 0 | 0 | 115.5 ms | 120.0 ms |
| qbf | families: classical auto j1 | focus | 20 | 15 | 5 | 10 | 5 | 0 | 0 | 0 | 122.2 ms | 321.21 s |
| wide-m1 | families: classical auto j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 198 µs | 631.6 ms |
| wide-m2 | families: classical auto j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 194 µs | 617.2 ms |
| wide-m3 | families: classical auto j1 | focus | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 168.4 ms | 10.93 s |
| wide-m4 | families: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 2.72 s | 47.60 s |
| mix | families: mix auto j1 | focus | 6 | 5 | 0 | 5 | 1 | 0 | 0 | 0 | 1.95 s | 238.70 s |
| counter | families: classical auto j1 | focus | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 84 µs | 128.7 ms |
| counter-over | families: classical auto j1 | focus | 4 | 0 | 0 | 0 | 1 | 3 | 0 | 0 | – | – |
| growing | families: classical auto j1 | focus | 4 | 0 | 0 | 0 | 0 | 3 | 1 | 0 | – | – |
| chain | families: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 273.6 ms | 3.17 s |
| additive | families: classical auto j1 | additive | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 72.0 ms | 621.1 ms |
| counter | intuitionistic: intuitionistic auto j1 | two-sided | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 51 µs | 16.5 ms |
| counter-over | intuitionistic: intuitionistic auto j1 | two-sided | 4 | 0 | 0 | 0 | 1 | 3 | 0 | 0 | – | – |
| chain | intuitionistic: intuitionistic auto j1 | two-sided | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 276.3 ms | 3.19 s |
| 3-partition-yes | intuitionistic: intuitionistic auto j1 | two-sided | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 1.2 ms | 18.3 ms |
| 3-partition-no | intuitionistic: intuitionistic auto j1 | two-sided | 1 | 1 | 0 | 1 | 0 | 0 | 0 | 0 | 25.51 s | 25.51 s |
| CLL/Non-theorems | lltp-classical: classical auto j1 | focus | 3 | 3 | 0 | 3 | 0 | 0 | 0 | 0 | 14 µs | 86 µs |
| CLL/misc | lltp-classical: classical auto j1 | focus, net | 14 | 8 | 8 | 0 | 1 | 5 | 0 | 0 | 16 µs | 128 µs |
| ILL/ILLTP-LCL-01 | lltp-classical: classical auto j1 | focus | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-LCL-cbn | lltp-classical: classical auto j1 | focus | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 32 µs | 32 µs |
| ILL/ILLTP-LCL-cbv | lltp-classical: classical auto j1 | focus | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-01 | lltp-classical: classical auto j1 | focus | 252 | 11 | 10 | 1 | 9 | 209 | 23 | 0 | 18 µs | 285 µs |
| ILL/ILLTP-SYJ-cbn | lltp-classical: classical auto j1 | focus | 252 | 34 | 13 | 21 | 10 | 186 | 22 | 0 | 21 µs | 801 µs |
| ILL/ILLTP-SYJ-cbv | lltp-classical: classical auto j1 | focus | 252 | 39 | 18 | 21 | 30 | 173 | 10 | 0 | 31 µs | 576.4 ms |
| ILL/ILLTP-SYN-01 | lltp-classical: classical auto j1 | focus | 19 | 11 | 4 | 7 | 0 | 8 | 0 | 0 | 12 µs | 153 µs |
| ILL/ILLTP-SYN-cbn | lltp-classical: classical auto j1 | focus, net | 19 | 16 | 6 | 10 | 0 | 3 | 0 | 0 | 12 µs | 272 µs |
| ILL/ILLTP-SYN-cbv | lltp-classical: classical auto j1 | focus | 19 | 16 | 7 | 9 | 0 | 3 | 0 | 0 | 15 µs | 282 µs |
| ILL/KLE-01 | lltp-classical: classical auto j1 | focus | 88 | 50 | 48 | 2 | 0 | 38 | 0 | 0 | 28 µs | 1.4 ms |
| ILL/KLE-IMP-CONJ/ALT | lltp-classical: classical auto j1 | focus, net | 27 | 27 | 27 | 0 | 0 | 0 | 0 | 0 | 17 µs | 439 µs |
| ILL/KLE-IMP-CONJ | lltp-classical: classical auto j1 | focus, net | 222 | 162 | 162 | 0 | 0 | 60 | 0 | 0 | 21 µs | 3.7 ms |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-classical: classical auto j1 | focus, net | 22 | 22 | 0 | 22 | 0 | 0 | 0 | 0 | 5 µs | 146 µs |
| ILL/KLE-cbn | lltp-classical: classical auto j1 | focus | 88 | 68 | 66 | 2 | 0 | 20 | 0 | 0 | 18 µs | 1.5 ms |
| ILL/KLE-cbv | lltp-classical: classical auto j1 | focus | 88 | 67 | 64 | 3 | 0 | 21 | 0 | 0 | 21 µs | 1.6 ms |
| ILL/Non-theorems | lltp-classical: classical auto j1 | focus | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | – | – |
| ILL/misc | lltp-classical: classical auto j1 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 17 µs | 78 µs |
| ILL/petri-nets/MCC | lltp-classical: classical auto j1 | focus | 2417 | 137 | 137 | 0 | 836 | 52 | 1392 | 0 | 805 µs | 16.03 s |
| ILL/ILLTP-LCL-01 | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-LCL-cbn | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 29 µs | 29 µs |
| ILL/ILLTP-LCL-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-01 | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 252 | 11 | 10 | 1 | 9 | 209 | 23 | 0 | 16 µs | 285 µs |
| ILL/ILLTP-SYJ-cbn | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 252 | 34 | 13 | 21 | 10 | 186 | 22 | 1 | 23 µs | 857 µs |
| ILL/ILLTP-SYJ-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 252 | 39 | 18 | 21 | 30 | 173 | 10 | 0 | 30 µs | 567.7 ms |
| ILL/ILLTP-SYN-01 | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 19 | 11 | 4 | 7 | 0 | 8 | 0 | 2 | 12 µs | 159 µs |
| ILL/ILLTP-SYN-cbn | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 19 | 16 | 6 | 10 | 0 | 3 | 0 | 3 | 12 µs | 278 µs |
| ILL/ILLTP-SYN-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 19 | 16 | 7 | 9 | 0 | 3 | 0 | 2 | 15 µs | 286 µs |
| ILL/KLE-01 | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 88 | 50 | 48 | 2 | 0 | 38 | 0 | 2 | 28 µs | 1.4 ms |
| ILL/KLE-IMP-CONJ/ALT | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 27 | 27 | 27 | 0 | 0 | 0 | 0 | 0 | 17 µs | 454 µs |
| ILL/KLE-IMP-CONJ | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 222 | 162 | 162 | 0 | 0 | 60 | 0 | 0 | 21 µs | 3.8 ms |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 22 | 22 | 0 | 22 | 0 | 0 | 0 | 0 | 6 µs | 160 µs |
| ILL/KLE-cbn | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 88 | 68 | 66 | 2 | 0 | 20 | 0 | 2 | 18 µs | 1.5 ms |
| ILL/KLE-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 88 | 67 | 64 | 3 | 0 | 21 | 0 | 2 | 22 µs | 1.6 ms |
| ILL/Non-theorems | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | – | – |
| ILL/misc | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 18 µs | 69 µs |
| ILL/petri-nets/MCC | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 2664 | 184 | 184 | 0 | 827 | 161 | 1492 | 0 | 554 µs | 23.32 s |
| wide-m1 | period-16: classical net j1 period 16 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 450.0 ms | 456.4 ms |
| wide-m2 | period-16: classical net j1 period 16 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 435.4 ms | 441.9 ms |
| 3-partition-mll-no | period-16: classical net j1 period 16 | net | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 21.46 s | 21.46 s |
| partition-yes | period-16: classical net j1 period 16 | net | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | – | – |
| partition-no | period-16: classical net j1 period 16 | net | 2 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | – | – |
| wide-m1 | period-1: classical net j1 period 1 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 1.37 s | 1.39 s |
| wide-m2 | period-1: classical net j1 period 1 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 1.37 s | 1.39 s |
| 3-partition-mll-no | period-1: classical net j1 period 1 | net | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 57.42 s | 59.24 s |
| partition-yes | period-1: classical net j1 period 1 | net | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 2.80 s | 2.80 s |
| partition-no | period-1: classical net j1 period 1 | net | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 3.54 s | 3.54 s |
| wide-m1 | period-2: classical net j1 period 2 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 872.9 ms | 885.3 ms |
| wide-m2 | period-2: classical net j1 period 2 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 856.9 ms | 869.5 ms |
| 3-partition-mll-no | period-2: classical net j1 period 2 | net | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 51.76 s | 53.18 s |
| partition-yes | period-2: classical net j1 period 2 | net | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 6.11 s | 6.11 s |
| partition-no | period-2: classical net j1 period 2 | net | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 8.25 s | 8.25 s |
| wide-m1 | period-8: classical net j1 period 8 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 513.6 ms | 520.9 ms |
| wide-m2 | period-8: classical net j1 period 8 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 503.0 ms | 510.3 ms |
| 3-partition-mll-no | period-8: classical net j1 period 8 | net | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 5.22 s | 5.22 s |
| partition-yes | period-8: classical net j1 period 8 | net | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | – | – |
| partition-no | period-8: classical net j1 period 8 | net | 2 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | – | – |

## partition-yes

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|
| partition-yes/4 | 2.4 ms ✓ | 2.76 s ✓ | 2.4 ms ✓ | > 60 s | 2.80 s ✓ | 6.11 s ✓ | > 60 s |
| partition-yes/5 | 182.6 ms ✓ | > 60 s | 191.7 ms ✓ |  |  |  |  |
| partition-yes/6 | 12.63 s ✓ | > 60 s | 12.94 s ✓ |  |  |  |  |
| partition-yes/7 |  |  | > 300 s |  |  |  |  |

## partition-no

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|
| partition-no/3 | 4.3 ms ✗ | 3.51 s ✗ | 4.4 ms ✗ | > 60 s | 3.54 s ✗ | 8.25 s ✗ | > 60 s |
| partition-no/4 | 113.8 ms ✗ | > 60 s | 115.5 ms ✗ | > 60 s | > 60 s | > 60 s | > 60 s |
| partition-no/5 |  |  | > 300 s |  |  |  |  |

## 3-partition-mll-yes

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 |
|---|--:|--:|--:|
| 3-partition-mll-yes/4 | 29 µs ✓ | 806 µs ✓ | 28 µs ✓ |
| 3-partition-mll-yes/6 | 32 µs ✓ | 115.1 ms ✓ | 30 µs ✓ |
| 3-partition-mll-yes/8 | 35 µs ✓ | 5.87 s ✓ | 31 µs ✓ |
| 3-partition-mll-yes/12 | 36 µs ✓ | > 60 s | 35 µs ✓ |

## 3-partition-mll-no

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|
| 3-partition-mll-no/4 | 17 µs ✗ | 1.84 s ✗ | 15 µs ✗ | 21.46 s ✗ | 1.82 s ✗ | 1.43 s ✗ | 5.22 s ✗ |
| 3-partition-mll-no/5 | 17 µs ✗ | 57.19 s ✗ | 15 µs ✗ | > 60 s | 57.42 s ✗ | 51.76 s ✗ | > 60 s |
| 3-partition-mll-no/6 | 16 µs ✗ | > 60 s | 14 µs ✗ |  |  |  |  |
| 3-partition-mll-no/8 | 17 µs ✗ | > 60 s | 15 µs ✗ |  |  |  |  |

## wide-m1

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|
| wide-m1/8 | 40 µs ✓ | 45 µs ✓ | 40 µs ✓ |  |  |  |  |
| wide-m1/16 | 752 µs ✓ | 114 µs ✓ | 102 µs ✓ |  |  |  |  |
| wide-m1/24 | 178.4 ms ✓ | 227 µs ✓ | 198 µs ✓ |  |  |  |  |
| wide-m1/32 | 45.61 s ✓ | 194 µs ✓ | 176 µs ✓ |  |  |  |  |
| wide-m1/256 | 1.0 ms ? wide | 9.3 ms ✓ | 8.7 ms ✓ | 6.4 ms ✓ | 19.4 ms ✓ | 12.4 ms ✓ | 7.3 ms ✓ |
| wide-m1/2048 | 51.1 ms ? wide | 639.3 ms ✓ | 622.3 ms ✓ | 450.0 ms ✓ | 1.37 s ✓ | 872.9 ms ✓ | 513.6 ms ✓ |

## wide-m2

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|
| wide-m2/8 | 38 µs ✓ | 46 µs ✓ | 42 µs ✓ |  |  |  |  |
| wide-m2/16 | 720 µs ✓ | 109 µs ✓ | 99 µs ✓ |  |  |  |  |
| wide-m2/24 | 172.8 ms ✓ | 220 µs ✓ | 194 µs ✓ |  |  |  |  |
| wide-m2/32 | 43.71 s ✓ | 192 µs ✓ | 172 µs ✓ |  |  |  |  |
| wide-m2/256 | 511 µs ? wide | 9.3 ms ✓ | 8.8 ms ✓ | 6.5 ms ✓ | 19.9 ms ✓ | 12.5 ms ✓ | 7.3 ms ✓ |
| wide-m2/2048 | 26.9 ms ? wide | 622.9 ms ✓ | 608.0 ms ✓ | 435.4 ms ✓ | 1.37 s ✓ | 856.9 ms ✓ | 503.0 ms ✓ |

## wide-m3

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 |
|---|--:|--:|--:|
| wide-m3/12 | 84 µs ✓ | 73 µs ✓ | 77 µs ✓ |
| wide-m3/24 | 171.0 ms ✓ | 218 µs ✓ | 168.4 ms ✓ |
| wide-m3/30 | 10.90 s ✓ | 163 µs ✓ | 10.76 s ✓ |
| wide-m3/36 |  |  | > 300 s |

## wide-m4

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 |
|---|--:|--:|--:|
| wide-m4/12 | 80 µs ✓ | 68 µs ✓ | 86 µs ✓ |
| wide-m4/24 | 172.8 ms ✓ | 207 µs ✓ | 170.0 ms ✓ |
| wide-m4/28 | 2.75 s ✓ | 267 µs ✓ | 2.72 s ✓ |
| wide-m4/32 |  |  | 44.71 s ✓ |

## additive

| problem | engines: classical focus j1 | families: classical auto j1 |
|---|--:|--:|
| additive/8 | 1.4 ms ✓ | 386 µs ✓ |
| additive/12 | 336.2 ms ✓ | 13.7 ms ✓ |
| additive/14 | 6.05 s ✓ | 72.0 ms ✓ |
| additive/16 | 16.72 s unknown | 535.1 ms ✓ |

## partition-table

| problem | engines: classical focus j1 | engines: classical net j1 |
|---|--:|--:|
| partition-table/1-1 | 33 µs ✓ | 36 µs ✓ |
| partition-table/1-3 | 119 µs ✗ | 370 µs ✗ |
| partition-table/2-1-1 | 149 µs ✓ | 613 µs ✓ |
| partition-table/1-1-4 | 2.1 ms ✗ | 39.3 ms ✗ |
| partition-table/2-2-1-1 | 2.0 ms ✓ | 50.8 ms ✓ |
| partition-table/1-2-5 | 5.8 ms ✗ | 3.67 s ✗ |
| partition-table/1-1-2-4 | 516 µs ✓ | 1.04 s ✓ |
| partition-table/1-1-1-5 | 60.9 ms ✗ | 7.47 s ✗ |
| partition-table/2-3-2-1 | 5.9 ms ✓ | 4.52 s ✓ |
| partition-table/3-3-3-1 | 111.6 ms ✗ | > 60 s |
| partition-table/1-2-3-4-5-5 | 23.31 s ✓ | > 60 s |
| partition-table/1-1-1-1-1-7 | 58.71 s ✗ | > 60 s |
| partition-table/2-2-2-2-2-2-9-1 | > 60 s | > 60 s |

## cancellation

| problem | engines: classical focus j1 |
|---|--:|
| cancellation/3-partition-4 | 51.20 s ✓ |

## 3-partition-yes

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 |
|---|--:|--:|
| 3-partition-yes/4 | 179 µs ✓ | 127 µs ✓ |
| 3-partition-yes/6 | 567 µs ✓ | 342 µs ✓ |
| 3-partition-yes/8 | 2.1 ms ✓ | 1.2 ms ✓ |
| 3-partition-yes/12 | 41.1 ms ✓ | 16.6 ms ✓ |

## 3-partition-no

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 |
|---|--:|--:|
| 3-partition-no/4 | 52.64 s ✗ | 25.51 s ✗ |
| 3-partition-no/5 | > 300 s |  |

## qbf

| problem | families: classical auto j1 |
|---|--:|
| qbf/8#0 | 566 µs ✗ |
| qbf/8#1 | 581 µs ✗ |
| qbf/8#2 | 800 µs ✓ |
| qbf/8#3 | 622 µs ✗ |
| qbf/12#0 | 122.2 ms ✓ |
| qbf/12#1 | 29.1 ms ✗ |
| qbf/12#2 | 45.7 ms ✓ |
| qbf/12#3 | 19.4 ms ✗ |
| qbf/16#0 | 8.10 s ✓ |
| qbf/16#1 | 1.46 s ✗ |
| qbf/16#2 | 2.09 s ✗ |
| qbf/16#3 | 6.72 s ✓ |
| qbf/20#0 | 88.37 s ✗ |
| qbf/20#1 | 124.58 s ✗ |
| qbf/20#2 | > 300 s |
| qbf/20#3 | 89.66 s ✗ |
| qbf/24#0 | > 300 s |
| qbf/24#1 | > 300 s |
| qbf/24#2 | > 300 s |
| qbf/24#3 | > 300 s |

## mix

| problem | families: mix auto j1 |
|---|--:|
| mix/4 | 267 µs ✗ |
| mix/6 | 21.0 ms ✗ |
| mix/8 | 1.95 s ✗ |
| mix/9 | 19.94 s ✗ |
| mix/10 | 216.79 s ✗ |
| mix/11 | > 300 s |

## counter

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 |
|---|--:|--:|
| counter/2 | 11 µs ✓ | 13 µs ✓ |
| counter/4 | 84 µs ✓ | 51 µs ✓ |
| counter/8 | 128.6 ms ✓ | 16.4 ms ✓ |
| counter/16 | > 300 s | > 300 s |

## counter-over

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 |
|---|--:|--:|
| counter-over/2 | 22 µs ? bound | 25 µs ? bound |
| counter-over/4 | 359 µs ? bound | 59 µs ? bound |
| counter-over/8 | 491.9 ms ? bound | 18.5 ms ? bound |
| counter-over/16 | > 300 s | > 300 s |

## growing

| problem | families: classical auto j1 |
|---|--:|
| growing/16 | 145 µs ? bound |
| growing/64 | 2.1 ms ? bound |
| growing/256 | 81.4 ms ? bound |
| growing/1024 | 581.1 ms ? depth |

## chain

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 |
|---|--:|--:|
| chain/16 | 242 µs ✓ | 269 µs ✓ |
| chain/64 | 22.3 ms ✓ | 22.7 ms ✓ |
| chain/128 | 273.6 ms ✓ | 276.3 ms ✓ |
| chain/256 | 2.87 s ✓ | 2.89 s ✓ |
