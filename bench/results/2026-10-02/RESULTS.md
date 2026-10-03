# Benchmark results

The baseline of 2026-10-02 on an Intel(R) Core(TM) Ultra X9 388H
(16 cores, 62 GB), release build, from
`bench/baseline.sh`, which writes this file, the CSV files beside it
(`bench/results/2026-10-02/`, one row per run) and a copy of this file as
`bench/RESULTS.md`, the latest baseline's. It started (once more for
every resumption), with the commit it measured:

- 2026-10-02 23:20, commit e5d930cf12dd (Record the target set on the engine of the second baseline), load average 0.24

Its last part took 8 h 4 min, and the scheduled jobs that ran
meanwhile were: none. The package throttled
523047 times for 7074 s in that part
(platform profile performance, governor powersave, energy preference balance_performance, turbo on).
The journal of `linlog-baseline` says every ten minutes what else used
a CPU. The sequential runs went in four
streams at once, each pinned to a performance core of its own; the
parallel runs had the machine to themselves. A time is the median of up to
three runs; `✓` is proved, `✗` refuted, `?` unknown (`bound`: the copy
bound, `wide`: a context too wide to split, `depth`: the recursion
limit), `>` a time limit reached, `MISMATCH` a verdict against the
problem's known one (for LLTP, the one its header claims). `linlog-bench
summary bench/results/2026-10-02/*.csv` prints the tables again.

## Solved within the time limit

Sequential problems that waited for a CPU for over 1 % of their time (another process slowed them down; `†` in the tables below): 5191.

| family | configuration | engines | problems | solved | proved | refuted | timeout | bound | other | mismatch | median solved | total solved |
|---|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| partition-yes | engines: classical focus j1 | focus | 5 | 5 | 5 | 0 | 0 | 0 | 0 | 0 | 130 µs | 589.1 ms |
| partition-yes | engines: classical net j1 | net | 5 | 1 | 1 | 0 | 4 | 0 | 0 | 0 | 2.83 s | 2.83 s |
| partition-no | engines: classical focus j1 | focus | 4 | 4 | 0 | 4 | 0 | 0 | 0 | 0 | 86.1 ms | 4.00 s |
| partition-no | engines: classical net j1 | net | 4 | 1 | 0 | 1 | 3 | 0 | 0 | 0 | 3.57 s | 3.57 s |
| 3-partition-mll-yes | engines: classical focus j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 42 µs | 169 µs |
| 3-partition-mll-yes | engines: classical net j1 | net | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 117.3 ms | 5.80 s |
| 3-partition-mll-no | engines: classical focus j1 | focus | 4 | 4 | 0 | 4 | 0 | 0 | 0 | 0 | 20 µs | 82 µs |
| 3-partition-mll-no | engines: classical net j1 | net | 4 | 2 | 0 | 2 | 2 | 0 | 0 | 0 | 57.68 s | 59.54 s |
| wide-m1 | engines: classical focus j1 | focus | 6 | 5 | 5 | 0 | 0 | 0 | 1 | 0 | 104 µs | 6.0 ms |
| wide-m1 | engines: classical net j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 207 µs | 647.2 ms |
| wide-m2 | engines: classical focus j1 | focus | 6 | 5 | 5 | 0 | 0 | 0 | 1 | 0 | 97 µs | 4.7 ms |
| wide-m2 | engines: classical net j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 205 µs | 645.1 ms |
| wide-m3 | engines: classical focus j1 | focus | 5 | 4 | 4 | 0 | 0 | 0 | 1 | 0 | 126 µs | 4.0 ms |
| wide-m3 | engines: classical net j1 | net | 5 | 5 | 5 | 0 | 0 | 0 | 0 | 0 | 213 µs | 659.5 ms |
| wide-m4 | engines: classical focus j1 | focus | 5 | 4 | 4 | 0 | 0 | 0 | 1 | 0 | 115 µs | 3.7 ms |
| wide-m4 | engines: classical net j1 | net | 5 | 5 | 5 | 0 | 0 | 0 | 0 | 0 | 278 µs | 683.8 ms |
| additive | engines: classical focus j1 | focus | 4 | 3 | 3 | 0 | 0 | 0 | 1 | 0 | 320.2 ms | 5.73 s |
| partition-table | engines: classical focus j1 | focus | 13 | 13 | 7 | 6 | 0 | 0 | 0 | 0 | 62 µs | 2.2 ms |
| partition-table | engines: classical net j1 | net | 13 | 9 | 5 | 4 | 4 | 0 | 0 | 0 | 55.2 ms | 17.21 s |
| cancellation | engines: classical focus j1 | focus | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 319 µs | 319 µs |
| 3-partition-yes | families: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 44 µs | 176 µs |
| 3-partition-no | families: classical auto j1 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 358 µs | 657 µs |
| 3-partition-mll-yes | families: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 39 µs | 154 µs |
| 3-partition-mll-no | families: classical auto j1 | focus | 4 | 4 | 0 | 4 | 0 | 0 | 0 | 0 | 20 µs | 77 µs |
| partition-yes | families: classical auto j1 | focus | 9 | 8 | 8 | 0 | 1 | 0 | 0 | 0 | 14.8 ms | 118.43 s |
| partition-no | families: classical auto j1 | focus | 7 | 5 | 0 | 5 | 2 | 0 | 0 | 0 | 465 µs | 3.95 s |
| qbf | families: classical auto j1 | focus | 36 | 34 | 6 | 28 | 1 | 0 | 1 | 0 | 57.2 ms | 1088.16 s |
| wide-m1 | families: classical auto j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 199 µs | 590.2 ms |
| wide-m2 | families: classical auto j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 191 µs | 580.4 ms |
| wide-m3 | families: classical auto j1 | focus | 7 | 6 | 6 | 0 | 0 | 0 | 1 | 0 | 139 µs | 58.9 ms |
| wide-m4 | families: classical auto j1 | focus | 8 | 7 | 7 | 0 | 0 | 0 | 1 | 0 | 119 µs | 55.3 ms |
| mix | families: mix auto j1 | focus | 6 | 5 | 0 | 5 | 1 | 0 | 0 | 0 | 1.55 s | 161.53 s |
| counter | families: classical auto j1 | focus | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 248 µs | 82.87 s |
| counter-over | families: classical auto j1 | focus | 6 | 4 | 0 | 4 | 0 | 2 | 0 | 0 | 110 µs | 523 µs |
| growing | families: classical auto j1 | focus | 4 | 0 | 0 | 0 | 0 | 3 | 1 | 0 | – | – |
| chain | families: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 40.2 ms | 366.7 ms |
| additive | families: classical auto j1 | additive | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 68.0 ms | 291.6 ms |
| counter | intuitionistic: intuitionistic auto j1 | two-sided | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 251 µs | 288.0 ms |
| counter-over | intuitionistic: intuitionistic auto j1 | two-sided | 6 | 4 | 0 | 4 | 0 | 2 | 0 | 0 | 107 µs | 533 µs |
| chain | intuitionistic: intuitionistic auto j1 | two-sided | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 42.4 ms | 395.4 ms |
| 3-partition-yes | intuitionistic: intuitionistic auto j1 | two-sided | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 47 µs | 184 µs |
| 3-partition-no | intuitionistic: intuitionistic auto j1 | two-sided | 1 | 1 | 0 | 1 | 0 | 0 | 0 | 0 | 293 µs | 293 µs |
| ILL/ILLTP-SYJ-01 | lltp-all-cores: intuitionistic auto j16 | two-sided | 31 | 0 | 0 | 0 | 2 | 13 | 16 | 0 | – | – |
| ILL/ILLTP-SYJ-cbn | lltp-all-cores: intuitionistic auto j16 | two-sided | 31 | 0 | 0 | 0 | 2 | 16 | 13 | 0 | – | – |
| ILL/ILLTP-SYJ-cbv | lltp-all-cores: intuitionistic auto j16 | two-sided | 31 | 2 | 2 | 0 | 29 | 0 | 0 | 0 | 2.62 s | 2.77 s |
| ILL/petri-nets/MCC | lltp-all-cores: intuitionistic auto j16 | two-sided | 1009 | 623 | 623 | 0 | 386 | 0 | 0 | 0 | 5.8 ms | 125.99 s |
| ILL/ILLTP-SYJ-01 | lltp-all-cores-generous: intuitionistic auto j16 | two-sided | 5 | 0 | 0 | 0 | 4 | 1 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-cbn | lltp-all-cores-generous: intuitionistic auto j16 | two-sided | 4 | 0 | 0 | 0 | 2 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-cbv | lltp-all-cores-generous: intuitionistic auto j16 | two-sided | 3 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | – | – |
| ILL/petri-nets/MCC | lltp-classical: classical auto j1 | focus | 3137 | 1468 | 1468 | 0 | 1653 | 2 | 14 | 0 | 1.4 ms | 272.50 s |
| ILL/misc | lltp-classical: classical auto j1 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 23 µs | 126 µs |
| ILL/Non-theorems | lltp-classical: classical auto j1 | focus | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | – | – |
| ILL/KLE-cbv | lltp-classical: classical auto j1 | focus | 88 | 67 | 64 | 3 | 0 | 21 | 0 | 0 | 27 µs | 1.9 ms |
| ILL/KLE-cbn | lltp-classical: classical auto j1 | focus | 88 | 68 | 66 | 2 | 0 | 20 | 0 | 0 | 45 µs | 3.0 ms |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-classical: classical auto j1 | focus, net | 22 | 22 | 0 | 22 | 0 | 0 | 0 | 0 | 7 µs | 182 µs |
| ILL/KLE-IMP-CONJ | lltp-classical: classical auto j1 | focus, net | 222 | 162 | 162 | 0 | 0 | 60 | 0 | 0 | 28 µs | 5.3 ms |
| ILL/KLE-IMP-CONJ/ALT | lltp-classical: classical auto j1 | focus, net | 27 | 27 | 27 | 0 | 0 | 0 | 0 | 0 | 20 µs | 693 µs |
| ILL/KLE-01 | lltp-classical: classical auto j1 | focus | 88 | 50 | 48 | 2 | 0 | 38 | 0 | 0 | 35 µs | 1.7 ms |
| ILL/ILLTP-SYN-cbv | lltp-classical: classical auto j1 | focus | 19 | 16 | 7 | 9 | 0 | 3 | 0 | 0 | 19 µs | 352 µs |
| ILL/ILLTP-SYN-cbn | lltp-classical: classical auto j1 | focus, net | 19 | 16 | 6 | 10 | 0 | 3 | 0 | 0 | 20 µs | 521 µs |
| ILL/ILLTP-SYN-01 | lltp-classical: classical auto j1 | focus | 19 | 11 | 4 | 7 | 0 | 8 | 0 | 0 | 16 µs | 186 µs |
| ILL/ILLTP-SYJ-cbv | lltp-classical: classical auto j1 | focus | 252 | 39 | 18 | 21 | 30 | 173 | 10 | 0 | 34 µs | 262.9 ms |
| ILL/ILLTP-SYJ-cbn | lltp-classical: classical auto j1 | focus | 252 | 35 | 13 | 22 | 10 | 185 | 22 | 0 | 62 µs | 2.1 ms |
| ILL/ILLTP-SYJ-01 | lltp-classical: classical auto j1 | focus | 252 | 11 | 10 | 1 | 10 | 209 | 22 | 0 | 21 µs | 314 µs |
| ILL/ILLTP-LCL-cbv | lltp-classical: classical auto j1 | focus | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-LCL-cbn | lltp-classical: classical auto j1 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 60 µs | 94 µs |
| ILL/ILLTP-LCL-01 | lltp-classical: classical auto j1 | focus | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| CLL/misc | lltp-classical: classical auto j1 | focus, net | 14 | 8 | 8 | 0 | 0 | 6 | 0 | 0 | 16 µs | 183 µs |
| CLL/Non-theorems | lltp-classical: classical auto j1 | focus | 3 | 3 | 0 | 3 | 0 | 0 | 0 | 0 | 17 µs | 99 µs |
| ILL/ILLTP-SYJ-01 | lltp-classical-generous: classical auto j1 | focus | 4 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-cbn | lltp-classical-generous: classical auto j1 | focus | 2 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-cbv | lltp-classical-generous: classical auto j1 | focus | 3 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | – | – |
| ILL/petri-nets/MCC | lltp-classical-generous: classical auto j1 | focus | 25 | 15 | 15 | 0 | 10 | 0 | 0 | 0 | 404 µs | 6.85 s |
| ILL/ILLTP-LCL-01 | lltp-copies-10: intuitionistic auto j1 | two-sided | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 48 µs | 48 µs |
| ILL/ILLTP-LCL-cbn | lltp-copies-10: intuitionistic auto j1 | two-sided | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 61 µs | 96 µs |
| ILL/ILLTP-LCL-cbv | lltp-copies-10: intuitionistic auto j1 | two-sided | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 55 µs | 55 µs |
| ILL/ILLTP-SYJ-01 | lltp-copies-10: intuitionistic auto j1 | two-sided | 209 | 37 | 35 | 2 | 107 | 65 | 0 | 1 | 1.9 ms | 11.27 s |
| ILL/ILLTP-SYJ-cbn | lltp-copies-10: intuitionistic auto j1 | two-sided | 209 | 75 | 39 | 36 | 88 | 46 | 0 | 1 | 141 µs | 13.64 s |
| ILL/ILLTP-SYJ-cbv | lltp-copies-10: intuitionistic auto j1 | two-sided | 209 | 83 | 56 | 27 | 98 | 28 | 0 | 4 | 459 µs | 21.96 s |
| ILL/ILLTP-SYN-01 | lltp-copies-10: intuitionistic auto j1 | two-sided | 8 | 5 | 3 | 2 | 0 | 3 | 0 | 0 | 111 µs | 741 µs |
| ILL/ILLTP-SYN-cbn | lltp-copies-10: intuitionistic auto j1 | two-sided | 8 | 5 | 3 | 2 | 0 | 3 | 0 | 0 | 51 µs | 287 µs |
| ILL/ILLTP-SYN-cbv | lltp-copies-10: intuitionistic auto j1 | two-sided | 8 | 5 | 3 | 2 | 0 | 3 | 0 | 0 | 37 µs | 189 µs |
| ILL/KLE-01 | lltp-copies-10: intuitionistic auto j1 | two-sided | 38 | 29 | 28 | 1 | 0 | 9 | 0 | 0 | 131 µs | 28.9 ms |
| ILL/KLE-IMP-CONJ | lltp-copies-10: intuitionistic auto j1 | two-sided | 60 | 60 | 60 | 0 | 0 | 0 | 0 | 0 | 92 µs | 7.5 ms |
| ILL/KLE-cbn | lltp-copies-10: intuitionistic auto j1 | two-sided | 38 | 34 | 28 | 6 | 0 | 4 | 0 | 5 | 70 µs | 3.6 ms |
| ILL/KLE-cbv | lltp-copies-10: intuitionistic auto j1 | two-sided | 38 | 32 | 28 | 4 | 0 | 6 | 0 | 3 | 48 µs | 6.0 ms |
| ILL/Non-theorems | lltp-copies-10: intuitionistic auto j1 | two-sided | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | – | – |
| ILL/petri-nets/MCC | lltp-copies-10: intuitionistic auto j1 | two-sided | 171 | 89 | 89 | 0 | 80 | 2 | 0 | 0 | 9.3 ms | 17.39 s |
| ILL/ILLTP-LCL-01 | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 51 µs | 51 µs |
| ILL/ILLTP-LCL-cbn | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 34 µs | 56 µs |
| ILL/ILLTP-LCL-cbv | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 51 µs | 51 µs |
| ILL/ILLTP-SYJ-01 | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 252 | 53 | 50 | 3 | 138 | 39 | 22 | 1 | 878 µs | 15.44 s |
| ILL/ILLTP-SYJ-cbn | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 252 | 98 | 60 | 38 | 102 | 30 | 22 | 1 | 115 µs | 10.90 s |
| ILL/ILLTP-SYJ-cbv | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 252 | 94 | 66 | 28 | 129 | 19 | 10 | 4 | 211 µs | 21.63 s |
| ILL/ILLTP-SYN-01 | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 19 | 16 | 7 | 9 | 0 | 3 | 0 | 2 | 29 µs | 943 µs |
| ILL/ILLTP-SYN-cbn | lltp-forward: intuitionistic auto j1 bias factors | net, two-sided | 19 | 16 | 6 | 10 | 0 | 3 | 0 | 3 | 16 µs | 334 µs |
| ILL/ILLTP-SYN-cbv | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 19 | 16 | 7 | 9 | 0 | 3 | 0 | 2 | 21 µs | 405 µs |
| ILL/KLE-01 | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 88 | 80 | 76 | 4 | 1 | 7 | 0 | 3 | 43 µs | 32.6 ms |
| ILL/KLE-IMP-CONJ/ALT | lltp-forward: intuitionistic auto j1 bias factors | net, two-sided | 27 | 27 | 27 | 0 | 0 | 0 | 0 | 0 | 21 µs | 542 µs |
| ILL/KLE-IMP-CONJ | lltp-forward: intuitionistic auto j1 bias factors | net, two-sided | 222 | 222 | 222 | 0 | 0 | 0 | 0 | 0 | 29 µs | 11.7 ms |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-forward: intuitionistic auto j1 bias factors | net, two-sided | 22 | 22 | 0 | 22 | 0 | 0 | 0 | 0 | 8 µs | 207 µs |
| ILL/KLE-cbn | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 88 | 84 | 76 | 8 | 0 | 4 | 0 | 7 | 24 µs | 3.8 ms |
| ILL/KLE-cbv | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 88 | 82 | 76 | 6 | 0 | 6 | 0 | 5 | 30 µs | 7.5 ms |
| ILL/Non-theorems | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | – | – |
| ILL/misc | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 23 µs | 77 µs |
| ILL/petri-nets/MCC | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 3137 | 1576 | 1576 | 0 | 1454 | 93 | 14 | 0 | 1.6 ms | 379.31 s |
| ILL/ILLTP-LCL-01 | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-LCL-cbn | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 62 µs | 95 µs |
| ILL/ILLTP-LCL-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-01 | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 252 | 11 | 10 | 1 | 10 | 209 | 22 | 0 | 22 µs | 338 µs |
| ILL/ILLTP-SYJ-cbn | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 252 | 35 | 13 | 22 | 10 | 185 | 22 | 1 | 67 µs | 2.2 ms |
| ILL/ILLTP-SYJ-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 252 | 39 | 18 | 21 | 30 | 173 | 10 | 0 | 37 µs | 277.4 ms |
| ILL/ILLTP-SYN-01 | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 19 | 11 | 4 | 7 | 0 | 8 | 0 | 2 | 16 µs | 196 µs |
| ILL/ILLTP-SYN-cbn | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 19 | 16 | 6 | 10 | 0 | 3 | 0 | 3 | 48 µs | 650 µs |
| ILL/ILLTP-SYN-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 19 | 16 | 7 | 9 | 0 | 3 | 0 | 2 | 20 µs | 360 µs |
| ILL/KLE-01 | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 88 | 50 | 48 | 2 | 0 | 38 | 0 | 2 | 33 µs | 1.7 ms |
| ILL/KLE-IMP-CONJ/ALT | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 27 | 27 | 27 | 0 | 0 | 0 | 0 | 0 | 21 µs | 736 µs |
| ILL/KLE-IMP-CONJ | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 222 | 162 | 162 | 0 | 0 | 60 | 0 | 0 | 28 µs | 5.3 ms |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 22 | 22 | 0 | 22 | 0 | 0 | 0 | 0 | 7 µs | 181 µs |
| ILL/KLE-cbn | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 88 | 68 | 66 | 2 | 0 | 20 | 0 | 2 | 45 µs | 2.8 ms |
| ILL/KLE-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 88 | 67 | 64 | 3 | 0 | 21 | 0 | 2 | 27 µs | 1.9 ms |
| ILL/Non-theorems | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | – | – |
| ILL/misc | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 23 µs | 111 µs |
| ILL/petri-nets/MCC | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 3137 | 1520 | 1520 | 0 | 1601 | 2 | 14 | 0 | 1.7 ms | 307.65 s |
| ILL/ILLTP-SYJ-01 | lltp-intuitionistic-generous: intuitionistic auto j1 | two-sided | 4 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-cbn | lltp-intuitionistic-generous: intuitionistic auto j1 | two-sided | 2 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-cbv | lltp-intuitionistic-generous: intuitionistic auto j1 | two-sided | 3 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | – | – |
| ILL/petri-nets/MCC | lltp-intuitionistic-generous: intuitionistic auto j1 | two-sided | 19 | 11 | 11 | 0 | 8 | 0 | 0 | 0 | 406 µs | 5.92 s |
| ILL/ILLTP-SYJ-01 | lltp-portfolio: intuitionistic auto j16 portfolio | two-sided | 31 | 0 | 0 | 0 | 2 | 13 | 16 | 0 | – | – |
| ILL/ILLTP-SYJ-cbn | lltp-portfolio: intuitionistic auto j16 portfolio | two-sided | 31 | 0 | 0 | 0 | 2 | 15 | 14 | 0 | – | – |
| ILL/ILLTP-SYJ-cbv | lltp-portfolio: intuitionistic auto j16 portfolio | two-sided | 31 | 2 | 2 | 0 | 29 | 0 | 0 | 0 | 2.66 s | 2.81 s |
| ILL/petri-nets/MCC | lltp-portfolio: intuitionistic auto j16 portfolio | two-sided | 1009 | 627 | 627 | 0 | 382 | 0 | 0 | 0 | 6.3 ms | 130.63 s |
| ILL/ILLTP-SYJ-01 | lltp-portfolio-generous: intuitionistic auto j16 portfolio | two-sided | 5 | 0 | 0 | 0 | 4 | 1 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-cbn | lltp-portfolio-generous: intuitionistic auto j16 portfolio | two-sided | 4 | 0 | 0 | 0 | 2 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-cbv | lltp-portfolio-generous: intuitionistic auto j16 portfolio | two-sided | 3 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | – | – |
| ILL/ILLTP-LCL-01 | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-LCL-cbn | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 36 µs | 36 µs |
| ILL/ILLTP-LCL-cbv | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-01 | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 252 | 11 | 10 | 1 | 10 | 209 | 22 | 0 | 22 µs | 322 µs |
| ILL/ILLTP-SYJ-cbn | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 252 | 34 | 13 | 21 | 10 | 186 | 22 | 1 | 28 µs | 984 µs |
| ILL/ILLTP-SYJ-cbv | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 252 | 39 | 18 | 21 | 30 | 173 | 10 | 0 | 35 µs | 284.0 ms |
| ILL/ILLTP-SYN-01 | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 19 | 11 | 4 | 7 | 0 | 8 | 0 | 2 | 16 µs | 193 µs |
| ILL/ILLTP-SYN-cbn | lltp-rarer: intuitionistic auto j1 bias rarer | net, two-sided | 19 | 16 | 6 | 10 | 0 | 3 | 0 | 3 | 14 µs | 333 µs |
| ILL/ILLTP-SYN-cbv | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 19 | 16 | 7 | 9 | 0 | 3 | 0 | 2 | 21 µs | 358 µs |
| ILL/KLE-01 | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 88 | 50 | 48 | 2 | 0 | 38 | 0 | 2 | 33 µs | 1.7 ms |
| ILL/KLE-IMP-CONJ/ALT | lltp-rarer: intuitionistic auto j1 bias rarer | net, two-sided | 27 | 27 | 27 | 0 | 0 | 0 | 0 | 0 | 20 µs | 551 µs |
| ILL/KLE-IMP-CONJ | lltp-rarer: intuitionistic auto j1 bias rarer | net, two-sided | 222 | 162 | 162 | 0 | 0 | 60 | 0 | 0 | 26 µs | 4.5 ms |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-rarer: intuitionistic auto j1 bias rarer | net, two-sided | 22 | 22 | 0 | 22 | 0 | 0 | 0 | 0 | 8 µs | 210 µs |
| ILL/KLE-cbn | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 88 | 68 | 66 | 2 | 0 | 20 | 0 | 2 | 22 µs | 1.8 ms |
| ILL/KLE-cbv | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 88 | 67 | 64 | 3 | 0 | 21 | 0 | 2 | 26 µs | 1.9 ms |
| ILL/Non-theorems | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | – | – |
| ILL/misc | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 22 µs | 83 µs |
| ILL/petri-nets/MCC | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 3137 | 442 | 442 | 0 | 2347 | 313 | 35 | 0 | 411 µs | 76.24 s |
| ILL/ILLTP-SYJ-01 | lltp-recursion: intuitionistic auto j1 | two-sided | 22 | 0 | 0 | 0 | 20 | 0 | 2 | 0 | – | – |
| ILL/ILLTP-SYJ-cbn | lltp-recursion: intuitionistic auto j1 | two-sided | 22 | 0 | 0 | 0 | 20 | 0 | 2 | 0 | – | – |
| ILL/ILLTP-SYJ-cbv | lltp-recursion: intuitionistic auto j1 | two-sided | 22 | 0 | 0 | 0 | 22 | 0 | 0 | 0 | – | – |
| ILL/petri-nets/MCC | lltp-recursion: intuitionistic auto j1 | two-sided | 929 | 280 | 280 | 0 | 632 | 0 | 17 | 0 | 16.3 ms | 92.14 s |
| ILL/petri-nets/MCC | lltp-recursion-generous: intuitionistic auto j1 | two-sided | 9 | 5 | 5 | 0 | 4 | 0 | 0 | 0 | 2.4 ms | 15.4 ms |
| 3-partition-no | long-1: classical auto j1 | focus | 1 | 1 | 0 | 1 | 0 | 0 | 0 | 0 | 393 µs | 393 µs |
| partition-no | long-1: classical auto j1 | focus | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 477 µs | 477 µs |
| partition-yes | long-1: classical auto j1 | focus | 2 | 1 | 1 | 0 | 1 | 0 | 0 | 0 | 230 µs | 230 µs |
| counter | long-1: classical auto j1 | focus | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 85.14 s | 85.14 s |
| mix | long-2: mix auto j1 | focus | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | – | – |
| wide-m3 | long-2: classical auto j1 | focus | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 157 µs | 157 µs |
| wide-m4 | long-2: classical auto j1 | focus | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 144 µs | 144 µs |
| qbf | long-2: classical auto j1 | focus | 2 | 1 | 0 | 1 | 0 | 0 | 1 | 0 | 161.2 ms | 161.2 ms |
| 3-partition-yes | parallel: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 58 µs | 219 µs |
| 3-partition-yes | parallel: classical auto j2 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 164 µs | 678 µs |
| 3-partition-yes | parallel: classical auto j4 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 198 µs | 762 µs |
| 3-partition-yes | parallel: classical auto j8 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 328 µs | 1.3 ms |
| 3-partition-yes | parallel: classical auto j16 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 540 µs | 2.1 ms |
| 3-partition-no | parallel: classical auto j1 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 470 µs | 865 µs |
| 3-partition-no | parallel: classical auto j2 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 483 µs | 922 µs |
| 3-partition-no | parallel: classical auto j4 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 354 µs | 696 µs |
| 3-partition-no | parallel: classical auto j8 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 467 µs | 898 µs |
| 3-partition-no | parallel: classical auto j16 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 720 µs | 1.4 ms |
| partition-yes | parallel: classical auto j1 | focus | 5 | 5 | 5 | 0 | 0 | 0 | 0 | 0 | 318 µs | 106.07 s |
| partition-yes | parallel: classical auto j2 | focus | 5 | 4 | 4 | 0 | 1 | 0 | 0 | 0 | 447 µs | 587.3 ms |
| partition-yes | parallel: classical auto j4 | focus | 5 | 4 | 4 | 0 | 1 | 0 | 0 | 0 | 498 µs | 609.0 ms |
| partition-yes | parallel: classical auto j8 | focus | 5 | 4 | 4 | 0 | 1 | 0 | 0 | 0 | 638 µs | 636.4 ms |
| partition-yes | parallel: classical auto j16 | focus | 5 | 4 | 4 | 0 | 1 | 0 | 0 | 0 | 849 µs | 653.3 ms |
| partition-no | parallel: classical auto j1 | focus | 4 | 3 | 0 | 3 | 1 | 0 | 0 | 0 | 604 µs | 4.01 s |
| partition-no | parallel: classical auto j2 | focus | 4 | 3 | 0 | 3 | 1 | 0 | 0 | 0 | 681 µs | 2.83 s |
| partition-no | parallel: classical auto j4 | focus | 4 | 3 | 0 | 3 | 1 | 0 | 0 | 0 | 518 µs | 1.73 s |
| partition-no | parallel: classical auto j8 | focus | 4 | 3 | 0 | 3 | 1 | 0 | 0 | 0 | 501 µs | 1.20 s |
| partition-no | parallel: classical auto j16 | focus | 4 | 3 | 0 | 3 | 1 | 0 | 0 | 0 | 840 µs | 849.2 ms |
| qbf | parallel: classical auto j1 | focus | 20 | 19 | 3 | 16 | 1 | 0 | 0 | 0 | 62.2 ms | 327.47 s |
| qbf | parallel: classical auto j2 | focus | 20 | 19 | 3 | 16 | 1 | 0 | 0 | 0 | 58.6 ms | 276.33 s |
| qbf | parallel: classical auto j4 | focus | 20 | 19 | 3 | 16 | 1 | 0 | 0 | 0 | 59.7 ms | 277.00 s |
| qbf | parallel: classical auto j8 | focus | 20 | 19 | 3 | 16 | 1 | 0 | 0 | 0 | 58.8 ms | 275.76 s |
| qbf | parallel: classical auto j16 | focus | 20 | 19 | 3 | 16 | 1 | 0 | 0 | 0 | 60.5 ms | 275.06 s |
| mix | parallel: mix auto j1 | focus | 4 | 2 | 0 | 2 | 2 | 0 | 0 | 0 | 13.94 s | 15.50 s |
| mix | parallel: mix auto j2 | focus | 4 | 2 | 0 | 2 | 2 | 0 | 0 | 0 | 17.40 s | 19.27 s |
| mix | parallel: mix auto j4 | focus | 4 | 2 | 0 | 2 | 2 | 0 | 0 | 0 | 17.88 s | 19.79 s |
| mix | parallel: mix auto j8 | focus | 4 | 2 | 0 | 2 | 2 | 0 | 0 | 0 | 18.57 s | 20.64 s |
| mix | parallel: mix auto j16 | focus | 4 | 2 | 0 | 2 | 2 | 0 | 0 | 0 | 21.18 s | 23.60 s |
| counter | parallel: classical auto j1 | focus | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 308 µs | 80.63 s |
| counter | parallel: classical auto j2 | focus | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 389 µs | 104.60 s |
| counter | parallel: classical auto j4 | focus | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 648 µs | 78.91 s |
| counter | parallel: classical auto j8 | focus | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 747 µs | 23.19 s |
| counter | parallel: classical auto j16 | focus | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 1.2 ms | 24.60 s |
| counter-over | parallel: classical auto j1 | focus | 6 | 4 | 0 | 4 | 0 | 2 | 0 | 0 | 116 µs | 661 µs |
| counter-over | parallel: classical auto j2 | focus | 6 | 4 | 0 | 4 | 0 | 2 | 0 | 0 | 165 µs | 816 µs |
| counter-over | parallel: classical auto j4 | focus | 6 | 4 | 0 | 4 | 0 | 2 | 0 | 0 | 274 µs | 1.3 ms |
| counter-over | parallel: classical auto j8 | focus | 6 | 4 | 0 | 4 | 0 | 2 | 0 | 0 | 454 µs | 2.2 ms |
| counter-over | parallel: classical auto j16 | focus | 6 | 4 | 0 | 4 | 0 | 2 | 0 | 0 | 759 µs | 3.1 ms |
| wide-m3 | parallel: classical auto j1 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 152 µs | 449 µs |
| wide-m3 | parallel: classical auto j2 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 269 µs | 741 µs |
| wide-m3 | parallel: classical auto j4 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 306 µs | 878 µs |
| wide-m3 | parallel: classical auto j8 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 424 µs | 1.2 ms |
| wide-m3 | parallel: classical auto j16 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 666 µs | 2.0 ms |
| wide-m4 | parallel: classical auto j1 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 175 µs | 503 µs |
| wide-m4 | parallel: classical auto j2 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 266 µs | 765 µs |
| wide-m4 | parallel: classical auto j4 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 284 µs | 880 µs |
| wide-m4 | parallel: classical auto j8 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 393 µs | 1.2 ms |
| wide-m4 | parallel: classical auto j16 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 638 µs | 2.0 ms |
| 3-partition-mll-no | parallel-net: classical net j1 | net | 3 | 2 | 0 | 2 | 1 | 0 | 0 | 0 | 51.56 s | 53.26 s |
| 3-partition-mll-no | parallel-net: classical net j2 | net | 3 | 2 | 0 | 2 | 1 | 0 | 0 | 0 | 26.63 s | 27.52 s |
| 3-partition-mll-no | parallel-net: classical net j4 | net | 3 | 2 | 0 | 2 | 1 | 0 | 0 | 0 | 14.46 s | 14.93 s |
| 3-partition-mll-no | parallel-net: classical net j8 | net | 3 | 2 | 0 | 2 | 1 | 0 | 0 | 0 | 7.92 s | 8.17 s |
| 3-partition-mll-no | parallel-net: classical net j16 | net | 3 | 2 | 0 | 2 | 1 | 0 | 0 | 0 | 4.51 s | 4.65 s |
| partition-table | parallel-net: classical net j1 | net | 13 | 9 | 5 | 4 | 4 | 0 | 0 | 0 | 60.3 ms | 16.55 s |
| partition-table | parallel-net: classical net j2 | net | 13 | 9 | 5 | 4 | 4 | 0 | 0 | 0 | 30.7 ms | 8.25 s |
| partition-table | parallel-net: classical net j4 | net | 13 | 10 | 6 | 4 | 3 | 0 | 0 | 0 | 16.7 ms | 4.24 s |
| partition-table | parallel-net: classical net j8 | net | 13 | 10 | 6 | 4 | 3 | 0 | 0 | 0 | 143.8 ms | 3.30 s |
| partition-table | parallel-net: classical net j16 | net | 13 | 10 | 6 | 4 | 3 | 0 | 0 | 0 | 72.5 ms | 2.14 s |
| wide-m1 | period-16: classical net j1 period 16 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 434.9 ms | 441.2 ms |
| wide-m2 | period-16: classical net j1 period 16 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 429.6 ms | 436.1 ms |
| 3-partition-mll-no | period-16: classical net j1 period 16 | net | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 20.98 s | 20.98 s |
| partition-yes | period-16: classical net j1 period 16 | net | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | – | – |
| partition-no | period-16: classical net j1 period 16 | net | 2 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | – | – |
| wide-m1 | period-1: classical net j1 period 1 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 1.35 s | 1.37 s |
| wide-m2 | period-1: classical net j1 period 1 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 1.38 s | 1.40 s |
| 3-partition-mll-no | period-1: classical net j1 period 1 | net | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 56.38 s | 58.24 s |
| partition-yes | period-1: classical net j1 period 1 | net | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 2.75 s | 2.75 s |
| partition-no | period-1: classical net j1 period 1 | net | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 3.49 s | 3.49 s |
| wide-m1 | period-2: classical net j1 period 2 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 860.9 ms | 873.4 ms |
| wide-m2 | period-2: classical net j1 period 2 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 855.4 ms | 867.8 ms |
| 3-partition-mll-no | period-2: classical net j1 period 2 | net | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 51.04 s | 52.49 s |
| partition-yes | period-2: classical net j1 period 2 | net | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 6.00 s | 6.00 s |
| partition-no | period-2: classical net j1 period 2 | net | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 7.82 s | 7.82 s |
| wide-m1 | period-8: classical net j1 period 8 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 494.5 ms | 501.8 ms |
| wide-m2 | period-8: classical net j1 period 8 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 491.0 ms | 498.2 ms |
| 3-partition-mll-no | period-8: classical net j1 period 8 | net | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 4.96 s | 4.96 s |
| partition-yes | period-8: classical net j1 period 8 | net | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | – | – |
| partition-no | period-8: classical net j1 period 8 | net | 2 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | – | – |

## partition-yes

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | long-1: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| partition-yes/4 | 60 µs ✓ | 2.83 s ✓ | 55 µs ✓ |  |  |  |  |  |  | > 60 s | 2.75 s ✓ | 6.00 s ✓ | > 60 s |
| partition-yes/5 | 77 µs ✓ | > 60 s | 76 µs ✓ |  | 97 µs ✓ | 207 µs ✓ | 260 µs ✓ | 399 µs ✓ | 703 µs ✓ |  |  |  |  |
| partition-yes/6 | 130 µs ✓ | > 60 s | 130 µs ✓ |  | 167 µs ✓ | 280 µs ✓ | 287 µs ✓ | 450 µs ✓ | 743 µs ✓ |  |  |  |  |
| partition-yes/12 | 15.2 ms ✓ | > 60 s | 14.8 ms ✓ |  |  |  |  |  |  |  |  |  |  |
| partition-yes/20 | 573.7 ms ✓ | > 60 s | 568.4 ms ✓ |  | 535.5 ms ✓ | 586.4 ms ✓ | 608.0 ms ✓ | 634.9 ms ✓ | 651.0 ms ✓ |  |  |  |  |
| partition-yes/7 |  |  | 213 µs ✓ | 230 µs ✓ | 318 µs ✓ | 447 µs ✓ | 498 µs ✓ | 638 µs ✓ | 849 µs ✓ |  |  |  |  |
| partition-yes/16 |  |  | 186.8 ms ✓ |  |  |  |  |  |  |  |  |  |  |
| partition-yes/24 |  |  | 117.66 s ✓ |  | 105.53 s ✓ | > 120 s | > 120 s | > 120 s | > 120 s |  |  |  |  |
| partition-yes/28 |  |  | > 300 s | > 1200 s |  |  |  |  |  |  |  |  |  |

## partition-no

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | long-1: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| partition-no/3 | 59 µs ✗ | 3.57 s ✗ | 59 µs ✗ |  |  |  |  |  |  | > 60 s | 3.49 s ✗ | 7.82 s ✗ | > 60 s |
| partition-no/4 | 138 µs ✗ | > 60 s | 141 µs ✗ |  | 170 µs ✗ | 272 µs ✗ | 242 µs ✗ | 392 µs ✗ | 721 µs ✗ | > 60 s | > 60 s | > 60 s | > 60 s |
| partition-no/9 | 86.1 ms ✗ | > 60 s | 84.9 ms ✗ |  |  |  |  |  |  |  |  |  |  |
| partition-no/12 | 3.91 s ✗ | > 60 s | 3.87 s ✗ |  | 4.01 s ✗ | 2.83 s ✗ | 1.73 s ✗ | 1.20 s ✗ | 847.6 ms ✗ |  |  |  |  |
| partition-no/5 |  |  | 465 µs ✗ | 477 µs ✗ | 604 µs ✗ | 681 µs ✗ | 518 µs ✗ | 501 µs ✗ | 840 µs ✗ |  |  |  |  |
| partition-no/14 |  |  | > 300 s |  | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |  |  |  |  |
| partition-no/15 |  |  | > 300 s | > 1200 s |  |  |  |  |  |  |  |  |  |

## 3-partition-mll-yes

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 |
|---|--:|--:|--:|
| 3-partition-mll-yes/4 | 40 µs ✓ | 806 µs ✓ | 36 µs ✓ |
| 3-partition-mll-yes/6 | 42 µs ✓ | 117.3 ms ✓ | 38 µs ✓ |
| 3-partition-mll-yes/8 | 45 µs ✓ | 5.68 s ✓ | 39 µs ✓ |
| 3-partition-mll-yes/12 | 42 µs ✓ | > 60 s | 41 µs ✓ |

## 3-partition-mll-no

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | parallel-net: classical net j1 | parallel-net: classical net j2 | parallel-net: classical net j4 | parallel-net: classical net j8 | parallel-net: classical net j16 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 3-partition-mll-no/4 | 20 µs ✗ | 1.86 s ✗ | 19 µs ✗ | 1.70 s ✗ | 886.3 ms ✗ | 463.9 ms ✗ | 254.6 ms ✗ | 139.7 ms ✗ | 20.98 s ✗ | 1.85 s ✗ | 1.45 s ✗ | 4.96 s ✗ |
| 3-partition-mll-no/5 | 20 µs ✗ | 57.68 s ✗ | 18 µs ✗ | 51.56 s ✗ | 26.63 s ✗ | 14.46 s ✗ | 7.92 s ✗ | 4.51 s ✗ | > 60 s | 56.38 s ✗ | 51.04 s ✗ | > 60 s |
| 3-partition-mll-no/6 | 22 µs ✗ | > 60 s | 20 µs ✗ | > 60 s | > 60 s | > 60 s | > 60 s | > 60 s |  |  |  |  |
| 3-partition-mll-no/8 | 20 µs ✗ | > 60 s | 20 µs ✗ |  |  |  |  |  |  |  |  |  |

## wide-m1

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|
| wide-m1/8 | 43 µs ✓ | 43 µs ✓ | 40 µs ✓ |  |  |  |  |
| wide-m1/16 | 68 µs ✓ | 110 µs ✓ | 98 µs ✓ |  |  |  |  |
| wide-m1/24 | 104 µs ✓ | 207 µs ✓ | 199 µs ✓ |  |  |  |  |
| wide-m1/32 | 160 µs ✓ | 179 µs ✓ | 166 µs ✓ |  |  |  |  |
| wide-m1/256 | 5.6 ms ✓ | 9.3 ms ✓ | 8.2 ms ✓ | 6.2 ms ✓ | 19.5 ms ✓ | 12.4 ms ✓ | 7.2 ms ✓ |
| wide-m1/2048 | 271.0 ms ? depth | 637.3 ms ✓ | 581.5 ms ✓ | 434.9 ms ✓ | 1.35 s ✓ | 860.9 ms ✓ | 494.5 ms ✓ |

## wide-m2

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|
| wide-m2/8 | 40 µs ✓ | 43 µs ✓ | 44 µs ✓ |  |  |  |  |
| wide-m2/16 | 66 µs ✓ | 103 µs ✓ | 97 µs ✓ |  |  |  |  |
| wide-m2/24 | 97 µs ✓ | 205 µs ✓ | 191 µs ✓ |  |  |  |  |
| wide-m2/32 | 141 µs ✓ | 179 µs ✓ | 165 µs ✓ |  |  |  |  |
| wide-m2/256 | 4.3 ms ✓ | 9.4 ms ✓ | 8.5 ms ✓ | 6.6 ms ✓ | 19.6 ms ✓ | 12.4 ms ✓ | 7.2 ms ✓ |
| wide-m2/2048 | 242.7 ms ? depth | 635.2 ms ✓ | 571.3 ms ✓ | 429.6 ms ✓ | 1.38 s ✓ | 855.4 ms ✓ | 491.0 ms ✓ |

## wide-m3

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | long-2: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| wide-m3/12 | 50 µs ✓ | 74 µs ✓ | 44 µs ✓ |  |  |  |  |  |  |
| wide-m3/24 | 94 µs ✓ | 213 µs ✓ | 88 µs ✓ |  | 115 µs ✓ | 198 µs ✓ | 254 µs ✓ | 331 µs ✓ | 666 µs ✓ |
| wide-m3/30 | 126 µs ✓ | 166 µs ✓ | 112 µs ✓ |  | 152 µs ✓ | 269 µs ✓ | 306 µs ✓ | 424 µs ✓ | 611 µs ✓ |
| wide-m3/256 | 3.7 ms ✓ | 9.7 ms ✓ | 3.3 ms ✓ |  |  |  |  |  |  |
| wide-m3/2048 | 215.8 ms ? depth | 649.4 ms ✓ | 204.3 ms ? depth |  |  |  |  |  |  |
| wide-m3/36 |  |  | 139 µs ✓ | 157 µs ✓ | 182 µs ✓ | 274 µs ✓ | 318 µs ✓ | 424 µs ✓ | 687 µs ✓ |
| wide-m3/1024 |  |  | 55.2 ms ✓ |  |  |  |  |  |  |

## wide-m4

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | long-2: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| wide-m4/12 | 53 µs ✓ | 73 µs ✓ | 49 µs ✓ |  |  |  |  |  |  |
| wide-m4/24 | 92 µs ✓ | 215 µs ✓ | 85 µs ✓ |  |  |  |  |  |  |
| wide-m4/28 | 115 µs ✓ | 278 µs ✓ | 104 µs ✓ |  | 149 µs ✓ | 229 µs ✓ | 267 µs ✓ | 369 µs ✓ | 699 µs ✓ |
| wide-m4/256 | 3.5 ms ✓ | 10.1 ms ✓ | 3.2 ms ✓ |  |  |  |  |  |  |
| wide-m4/2048 | 203.7 ms ? depth | 673.2 ms ✓ | 191.8 ms ? depth |  |  |  |  |  |  |
| wide-m4/32 |  |  | 119 µs ✓ |  | 179 µs ✓ | 266 µs ✓ | 284 µs ✓ | 393 µs ✓ | 620 µs ✓ |
| wide-m4/36 |  |  | 137 µs ✓ | 144 µs ✓ | 175 µs ✓ | 270 µs ✓ | 329 µs ✓ | 412 µs ✓ | 638 µs ✓ |
| wide-m4/1024 |  |  | 51.6 ms ✓ |  |  |  |  |  |  |

## additive

| problem | engines: classical focus j1 | families: classical auto j1 |
|---|--:|--:|
| additive/8 | 1.5 ms ✓ | 378 µs ✓ |
| additive/12 | 320.2 ms ✓ | 13.8 ms ✓ |
| additive/14 | 5.41 s ✓ | 68.0 ms ✓ |
| additive/16 | 9.95 s unknown | 209.5 ms ✓ |

## partition-table

| problem | engines: classical focus j1 | engines: classical net j1 | parallel-net: classical net j1 | parallel-net: classical net j2 | parallel-net: classical net j4 | parallel-net: classical net j8 | parallel-net: classical net j16 |
|---|--:|--:|--:|--:|--:|--:|--:|
| partition-table/1-1 | 33 µs ✓ | 34 µs ✓ | 40 µs ✓ | 245 µs ✓ | 253 µs ✓ | 312 µs ✓ | 505 µs ✓ |
| partition-table/1-3 | 29 µs ✗ | 372 µs ✗ | 472 µs ✗ | 554 µs ✗ | 1.3 ms ✗ | 1.4 ms ✗ | 1.6 ms ✗ |
| partition-table/2-1-1 | 45 µs ✓ | 644 µs ✓ | 797 µs ✓ | 931 µs ✓ | 4.2 ms ✓ | 4.3 ms ✓ | 4.4 ms ✓ |
| partition-table/1-1-4 | 56 µs ✗ | 40.7 ms ✗ | 47.5 ms ✗ | 25.6 ms ✗ | 13.3 ms ✗ | 7.9 ms ✗ | 5.3 ms ✗ |
| partition-table/2-2-1-1 | 62 µs ✓ | 55.2 ms ✓ | 60.3 ms ✓ | 30.7 ms ✓ | 16.7 ms ✓ | 9.4 ms ✓ | 7.1 ms ✓ |
| partition-table/1-2-5 | 59 µs ✗ | 3.75 s ✗ | 3.92 s ✗ | 1.92 s ✗ | 962.7 ms ✗ | 545.7 ms ✗ | 297.9 ms ✗ |
| partition-table/1-1-2-4 | 48 µs ✓ | 1.10 s ✓ | 1.04 s ✓ | 522.5 ms ✓ | 265.7 ms ✓ | 143.8 ms ✓ | 72.5 ms ✓ |
| partition-table/1-1-1-5 | 149 µs ✗ | 7.59 s ✗ | 7.11 s ✗ | 3.63 s ✗ | 1.92 s ✗ | 1.08 s ✗ | 598.5 ms ✗ |
| partition-table/2-3-2-1 | 68 µs ✓ | 4.67 s ✓ | 4.38 s ✓ | 2.13 s ✓ | 1.06 s ✓ | 611.1 ms ✓ | 316.1 ms ✓ |
| partition-table/3-3-3-1 | 143 µs ✗ | > 60 s | > 60 s | > 60 s | > 60 s | > 60 s | > 60 s |
| partition-table/1-2-3-4-5-5 | 80 µs ✓ | > 60 s | > 60 s | > 60 s | 3.4 ms ✓ | > 60 s | > 60 s |
| partition-table/1-1-1-1-1-7 | 1.3 ms ✗ | > 60 s | > 60 s | > 60 s | > 60 s | > 60 s | > 60 s |
| partition-table/2-2-2-2-2-2-9-1 | 124 µs ✓ | > 60 s | > 60 s | > 60 s | > 60 s | 897.2 ms ✓ | 835.8 ms ✓ |

## cancellation

| problem | engines: classical focus j1 |
|---|--:|
| cancellation/3-partition-4 | 319 µs ✓ |

## 3-partition-yes

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|
| 3-partition-yes/4 | 41 µs ✓ | 40 µs ✓ | 48 µs ✓ | 209 µs ✓ | 198 µs ✓ | 328 µs ✓ | 506 µs ✓ |
| 3-partition-yes/6 | 43 µs ✓ | 46 µs ✓ | 58 µs ✓ | 142 µs ✓ | 179 µs ✓ | 287 µs ✓ | 452 µs ✓ |
| 3-partition-yes/8 | 44 µs ✓ | 47 µs ✓ | 54 µs ✓ | 164 µs ✓ | 183 µs ✓ | 352 µs ✓ | 601 µs ✓ |
| 3-partition-yes/12 | 48 µs ✓ | 51 µs ✓ | 59 µs ✓ | 163 µs ✓ | 202 µs ✓ | 308 µs ✓ | 540 µs ✓ |

## 3-partition-no

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 | long-1: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| 3-partition-no/4 | 299 µs ✗ | 293 µs ✗ |  | 395 µs ✗ | 439 µs ✗ | 342 µs ✗ | 467 µs ✗ | 720 µs ✗ |
| 3-partition-no/5 | 358 µs ✗ |  | 393 µs ✗ | 470 µs ✗ | 483 µs ✗ | 354 µs ✗ | 431 µs ✗ | 689 µs ✗ |

## qbf

| problem | families: classical auto j1 | long-2: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|
| qbf/8#0 | 123 µs ✗ |  |  |  |  |  |  |
| qbf/8#1 | 161 µs ✗ |  |  |  |  |  |  |
| qbf/8#2 | 184 µs ✓ |  |  |  |  |  |  |
| qbf/8#3 | 141 µs ✗ |  |  |  |  |  |  |
| qbf/12#0 | 1.8 ms ✓ |  |  |  |  |  |  |
| qbf/12#1 | 502 µs ✗ |  |  |  |  |  |  |
| qbf/12#2 | 925 µs ✓ |  |  |  |  |  |  |
| qbf/12#3 | 371 µs ✗ |  |  |  |  |  |  |
| qbf/16#0 | 10.1 ms ✓ |  | 13.1 ms ✓ | 14.7 ms ✓ | 14.3 ms ✓ | 14.0 ms ✓ | 14.9 ms ✓ |
| qbf/16#1 | 1.5 ms ✗ |  | 1.9 ms ✗ | 2.1 ms ✗ | 1.7 ms ✗ | 1.9 ms ✗ | 2.4 ms ✗ |
| qbf/16#2 | 2.4 ms ✗ |  | 3.3 ms ✗ | 3.6 ms ✗ | 3.7 ms ✗ | 3.3 ms ✗ | 3.4 ms ✗ |
| qbf/16#3 | 7.7 ms ✓ |  | 10.3 ms ✓ | 12.3 ms ✓ | 12.3 ms ✓ | 12.6 ms ✓ | 13.2 ms ✓ |
| qbf/20#0 | 7.3 ms ✗ |  | 9.4 ms ✗ | 9.0 ms ✗ | 8.7 ms ✗ | 8.6 ms ✗ | 7.8 ms ✗ |
| qbf/20#1 | 12.0 ms ✗ |  | 16.1 ms ✗ | 14.7 ms ✗ | 15.0 ms ✗ | 13.8 ms ✗ | 12.4 ms ✗ |
| qbf/20#2 | 49.8 ms ✓ |  | 60.2 ms ✓ | 103.0 ms ✓ | 100.4 ms ✓ | 100.1 ms ✓ | 103.9 ms ✓ |
| qbf/20#3 | 7.1 ms ✗ |  | 9.2 ms ✗ | 7.7 ms ✗ | 8.0 ms ✗ | 7.0 ms ✗ | 7.5 ms ✗ |
| qbf/24#0 | 163.5 ms ✗ | 161.2 ms ✗ | 169.3 ms ✗ | 201.0 ms ✗ | 194.5 ms ✗ | 196.9 ms ✗ | 214.4 ms ✗ |
| qbf/24#1 | 58.0 ms ✗ |  | 62.6 ms ✗ | 58.6 ms ✗ | 59.7 ms ✗ | 58.5 ms ✗ | 60.5 ms ✗ |
| qbf/24#2 | 38.7 ms ✗ |  | 44.7 ms ✗ | 43.7 ms ✗ | 37.6 ms ✗ | 39.0 ms ✗ | 40.2 ms ✗ |
| qbf/24#3 | 57.2 ms ✗ |  | 62.2 ms ✗ | 58.2 ms ✗ | 58.6 ms ✗ | 58.8 ms ✗ | 58.9 ms ✗ |
| qbf/32#0 | 875.9 ms ✗ |  |  |  |  |  |  |
| qbf/32#1 | 943.7 ms ✗ |  |  |  |  |  |  |
| qbf/32#2 | 2.04 s ✗ |  |  |  |  |  |  |
| qbf/32#3 | 778.5 ms ✗ |  |  |  |  |  |  |
| qbf/40#0 | 17.53 s ✗ |  | 16.61 s ✗ | 14.07 s ✗ | 13.95 s ✗ | 13.93 s ✗ | 14.33 s ✗ |
| qbf/40#1 | 20.18 s ✗ |  | 18.54 s ✗ | 15.30 s ✗ | 15.25 s ✗ | 15.32 s ✗ | 15.24 s ✗ |
| qbf/40#2 | 23.34 s ✗ |  | 21.36 s ✗ | 17.69 s ✗ | 17.64 s ✗ | 17.64 s ✗ | 17.72 s ✗ |
| qbf/40#3 | 18.29 s ✗ |  | 16.91 s ✗ | 15.63 s ✗ | 15.46 s ✗ | 15.53 s ✗ | 15.47 s ✗ |
| qbf/44#0 | 77.60 s ✗ |  | 72.46 s ✗ | 54.94 s ✗ | 55.96 s ✗ | 55.39 s ✗ | 54.98 s ✗ |
| qbf/44#1 | 79.01 s ✗ |  | 72.68 s ✗ | 60.47 s ✗ | 60.23 s ✗ | 59.90 s ✗ | 59.98 s ✗ |
| qbf/44#2 | 115.85 s ✗ |  | 108.46 s ✗ | 97.70 s ✗ | 97.99 s ✗ | 97.53 s ✗ | 96.80 s ✗ |
| qbf/44#3 | 218.72 s ✗ |  | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |
| qbf/48#0 | 290.27 s unknown | 301.01 s unknown |  |  |  |  |  |
| qbf/48#1 | 282.04 s ✗ |  |  |  |  |  |  |
| qbf/48#2 | 230.53 s ✗ |  |  |  |  |  |  |
| qbf/48#3 | > 300 s |  |  |  |  |  |  |

## mix

| problem | families: mix auto j1 | long-2: mix auto j1 | parallel: mix auto j1 | parallel: mix auto j2 | parallel: mix auto j4 | parallel: mix auto j8 | parallel: mix auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|
| mix/4 | 302 µs ✗ |  |  |  |  |  |  |
| mix/6 | 18.4 ms ✗ |  |  |  |  |  |  |
| mix/8 | 1.55 s ✗ |  | 1.56 s ✗ | 1.87 s ✗ | 1.91 s ✗ | 2.07 s ✗ | 2.43 s ✗ |
| mix/9 | 14.10 s ✗ |  | 13.94 s ✗ | 17.40 s ✗ | 17.88 s ✗ | 18.57 s ✗ | 21.18 s ✗ |
| mix/10 | 145.85 s ✗ |  | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |
| mix/11 | > 300 s | > 1200 s | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |

## counter

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 | long-1: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| counter/2 | 51 µs ✓ | 50 µs ✓ |  | 65 µs ✓ | 109 µs ✓ | 156 µs ✓ | 243 µs ✓ | 465 µs ✓ |
| counter/4 | 65 µs ✓ | 61 µs ✓ |  | 66 µs ✓ | 99 µs ✓ | 161 µs ✓ | 274 µs ✓ | 559 µs ✓ |
| counter/8 | 87 µs ✓ | 88 µs ✓ |  | 98 µs ✓ | 138 µs ✓ | 300 µs ✓ | 419 µs ✓ | 691 µs ✓ |
| counter/16 | 248 µs ✓ | 251 µs ✓ | 280 µs ✓ | 308 µs ✓ | 389 µs ✓ | 648 µs ✓ | 747 µs ✓ | 1.2 ms ✓ |
| counter/32 | 2.18 s ✓ † | 19.9 ms ✓ † |  | 2.18 s ✓ | 2.70 s ✓ | 2.02 s ✓ | 629.7 ms ✓ | 700.0 ms ✓ |
| counter/64 | 80.68 s ✓ | 267.7 ms ✓ † | 85.14 s ✓ | 78.45 s ✓ | 101.90 s ✓ | 76.89 s ✓ | 22.55 s ✓ | 23.90 s ✓ |

## counter-over

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|
| counter-over/2 | 55 µs ✗ | 54 µs ✗ | 65 µs ✗ | 93 µs ✗ | 150 µs ✗ | 240 µs ✗ | 510 µs ✗ |
| counter-over/4 | 59 µs ✗ | 66 µs ✗ | 77 µs ✗ | 128 µs ✗ | 171 µs ✗ | 268 µs ✗ | 568 µs ✗ |
| counter-over/8 | 110 µs ✗ | 107 µs ✗ | 116 µs ✗ | 165 µs ✗ | 274 µs ✗ | 454 µs ✗ | 759 µs ✗ |
| counter-over/16 | 299 µs ✗ | 306 µs ✗ | 403 µs ✗ | 430 µs ✗ | 731 µs ✗ | 1.2 ms ✗ | 1.2 ms ✗ |
| counter-over/32 | 2.22 s ? bound | 20.3 ms ? bound † | 2.28 s ? bound | 3.15 s ? bound | 2.52 s ? bound | 2.72 s ? bound | 2.87 s ? bound |
| counter-over/64 | 85.03 s ? bound | 268.5 ms ? bound † | 82.99 s ? bound | 112.73 s ? bound | 86.58 s ? bound | 95.97 s ? bound | 103.02 s ? bound |

## growing

| problem | families: classical auto j1 |
|---|--:|
| growing/16 | 342 µs ? bound |
| growing/64 | 1.6 ms ? bound |
| growing/256 | 41.8 ms ? bound |
| growing/1024 | 256.8 ms ? depth |

## chain

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 |
|---|--:|--:|
| chain/16 | 151 µs ✓ | 159 µs ✓ |
| chain/64 | 5.1 ms ✓ | 5.4 ms ✓ |
| chain/128 | 40.2 ms ✓ | 42.4 ms ✓ |
| chain/256 | 321.3 ms ✓ | 347.4 ms ✓ |
